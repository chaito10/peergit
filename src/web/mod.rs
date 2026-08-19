pub mod html;
pub mod api;

use crate::config::FossilP2pConfig;
use crate::home::Home;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub struct WebState {
    pub home: Home,
    pub config: FossilP2pConfig,
}

pub async fn start_web_server(
    state: Arc<WebState>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("web dashboard on http://localhost:{port}");

    loop {
        let (stream, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                tracing::debug!("web connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    state: Arc<WebState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        let resp = b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
        writer.write_all(resp).await?;
        return Ok(());
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();
    drop(parts);

    let mut header_buf = String::new();
    let mut content_len = 0usize;
    loop {
        header_buf.clear();
        if reader.read_line(&mut header_buf).await? > 0 {
            if header_buf.trim().is_empty() {
                break;
            }
            if let Some(val) = header_buf.trim().strip_prefix("Content-Length:") {
                content_len = val.trim().parse().unwrap_or(0);
            }
        } else {
            break;
        }
    }

    let (status, content_type, body) = match (method.as_str(), path.as_str()) {
        ("GET", "/") => (
            "200 OK",
            "text/html; charset=utf-8",
            html::dashboard_html().as_bytes().to_vec(),
        ),
        ("GET", "/api/status") => {
            let json = api::node_status(&state)?;
            ("200 OK", "application/json", json.into_bytes())
        }
        ("GET", "/api/peers") => {
            let json = api::list_peers(&state)?;
            ("200 OK", "application/json", json.into_bytes())
        }
        ("GET", "/api/repos") => {
            let json = api::list_repos(&state)?;
            ("200 OK", "application/json", json.into_bytes())
        }
        ("POST", "/api/peers") => {
            let mut body_buf = vec![0u8; content_len];
            if content_len > 0 && content_len < 1_048_576 {
                tokio::io::AsyncReadExt::read_exact(&mut reader, &mut body_buf).await?;
            }
            let json = api::add_peer(&state, &body_buf)?;
            ("200 OK", "application/json", json.into_bytes())
        }
        ("POST", "/api/sync") => {
            let json = api::trigger_sync(&state)?;
            ("200 OK", "application/json", json.into_bytes())
        }
        _ => (
            "404 Not Found",
            "text/plain",
            b"not found".to_vec(),
        ),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        body.len()
    );
    writer.write_all(response.as_bytes()).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;

    Ok(())
}
