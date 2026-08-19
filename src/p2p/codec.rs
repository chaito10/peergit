use async_trait::async_trait;
use futures::{AsyncReadExt, AsyncWriteExt};
use libp2p::request_response;
use std::io;

const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

pub const XFER_PROTOCOL: &str = "/peergit/xfer/1.0";

#[derive(Debug, Clone, Default)]
pub struct FossilCodec;

#[async_trait]
impl request_response::Codec for FossilCodec {
    type Protocol = String;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(
        &mut self,
        _protocol: &String,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        read_length_prefixed(io).await
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &String,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &req).await
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &String,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        read_length_prefixed(io).await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &String,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &res).await
    }
}

async fn read_length_prefixed<T: futures::AsyncRead + Unpin>(
    io: &mut T,
) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    io.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {len} bytes (max {MAX_MESSAGE_SIZE})"),
        ));
    }

    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn write_length_prefixed<T: futures::AsyncWrite + Unpin>(
    io: &mut T,
    data: &[u8],
) -> io::Result<()> {
    let len = data.len() as u32;
    io.write_all(&len.to_be_bytes()).await?;
    io.write_all(data).await?;
    Ok(())
}
