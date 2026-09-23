use crate::crypto::{Keypair, PublicKey};
use crate::error::{FossilP2pError, Result};
use crate::protocol::{
    decode_envelope, encode_envelope, Envelope, MessageType, Operation, XferRequest, XferResponse,
    PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 4 * 1024 * 1024;
pub const MAX_AGE_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkedRequest {
    pub rid: String,
    pub operation: Operation,
    pub total_size: u64,
    pub checksum: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkedResponse {
    pub rid: String,
    pub ok: bool,
    pub error: Option<String>,
    pub total_size: u64,
    pub checksum: [u8; 32],
}

/// Build a signed xfer request envelope for the given operation and payload.
pub fn build_xfer_request(
    keypair: &Keypair,
    rid: &str,
    operation: Operation,
    payload: Vec<u8>,
) -> Result<(Vec<u8>, [u8; 16])> {
    let xfer = XferRequest {
        rid: rid.to_string(),
        operation,
        payload,
    };
    let payload_bytes = serde_json::to_vec(&xfer)?;
    let env = Envelope::new(MessageType::XferRequest, keypair, payload_bytes);
    let request_id = env.request_id;
    Ok((encode_envelope(&env)?, request_id))
}

/// Parse and verify an inbound xfer request envelope.
pub struct XferParseResult {
    pub env: Envelope,
    pub xfer: XferRequest,
}

pub fn parse_xfer_envelope(bytes: &[u8], expected_rid: Option<&str>) -> Result<XferParseResult> {
    if bytes.len() > crate::protocol::MAX_PAYLOAD_SIZE + 4096 {
        return Err(FossilP2pError::Protocol("envelope too large".into()));
    }
    let env = decode_envelope(bytes)?;

    if env.version != PROTOCOL_VERSION {
        return Err(FossilP2pError::Protocol(format!(
            "protocol version mismatch: got {} want {}",
            env.version, PROTOCOL_VERSION
        )));
    }
    if env.message_type != MessageType::XferRequest {
        return Err(FossilP2pError::Protocol("expected xfer request".into()));
    }
    if !env.is_fresh(MAX_AGE_SECS) {
        return Err(FossilP2pError::Protocol("envelope timestamp stale".into()));
    }
    if !env.verify() {
        return Err(FossilP2pError::Authorization(
            "invalid request signature".into(),
        ));
    }

    let xfer: XferRequest = serde_json::from_slice(&env.payload)?;

    if let Some(rid) = expected_rid {
        if xfer.rid != rid {
            return Err(FossilP2pError::Authorization(format!(
                "requested repo {} != expected {rid}",
                xfer.rid
            )));
        }
    }

    Ok(XferParseResult { env, xfer })
}

/// Build a signed xfer response envelope.
pub fn build_xfer_response(
    keypair: &Keypair,
    request_id: [u8; 16],
    rid: &str,
    payload: Vec<u8>,
) -> Result<Vec<u8>> {
    let xfer = XferResponse {
        rid: rid.to_string(),
        ok: true,
        error: None,
        payload,
    };
    let payload_bytes = serde_json::to_vec(&xfer)?;

    let response = Envelope::with_request_id(
        MessageType::XferResponse,
        keypair,
        request_id,
        payload_bytes,
    );
    encode_envelope(&response)
}

/// Build an error xfer response envelope.
pub fn build_xfer_error(
    keypair: &Keypair,
    request_id: [u8; 16],
    rid: &str,
    error: &str,
) -> Result<Vec<u8>> {
    let xfer = XferResponse {
        rid: rid.to_string(),
        ok: false,
        error: Some(error.to_string()),
        payload: vec![],
    };
    let payload_bytes = serde_json::to_vec(&xfer)?;

    let response = Envelope::with_request_id(
        MessageType::XferResponse,
        keypair,
        request_id,
        payload_bytes,
    );
    encode_envelope(&response)
}

/// Verify and decode a response envelope, returning the XferResponse.
pub fn parse_xfer_response(bytes: &[u8]) -> Result<(Envelope, XferResponse)> {
    let env = decode_envelope(bytes)?;
    if env.version != PROTOCOL_VERSION {
        return Err(FossilP2pError::Protocol(format!(
            "protocol version mismatch: got {} want {}",
            env.version, PROTOCOL_VERSION
        )));
    }
    if env.message_type != MessageType::XferResponse {
        return Err(FossilP2pError::Protocol("expected xfer response".into()));
    }
    if !env.is_fresh(MAX_AGE_SECS) {
        return Err(FossilP2pError::Protocol("response timestamp stale".into()));
    }
    if !env.verify() {
        return Err(FossilP2pError::Authorization(
            "invalid response signature".into(),
        ));
    }
    let xfer: XferResponse = serde_json::from_slice(&env.payload)?;
    Ok((env, xfer))
}

pub fn split_into_chunks(data: &[u8], chunk_size: usize) -> Vec<&[u8]> {
    data.chunks(chunk_size).collect()
}

pub fn compute_sha256(data: &[u8]) -> [u8; 32] {
    crate::crypto::sha256(data)
}

/// Verify that the remote peer is authorized to access the given repository.
pub fn check_peer_authorized(
    repo_owner_did: &str,
    visibility: &crate::identity::Visibility,
    peer_key: &PublicKey,
    operation: Operation,
) -> bool {
    match visibility {
        crate::identity::Visibility::Public => true,
        crate::identity::Visibility::Private { allow } => {
            allow.iter().any(|k| k == &peer_key.to_hex() || k == &peer_key.to_multibase())
        }
        crate::identity::Visibility::Protected => {
            if operation.requires_write() {
                repo_owner_did == peer_key.to_did_key()
            } else {
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Keypair;

    #[test]
    fn build_and_parse_request() {
        let kp = Keypair::generate();
        let (bytes, req_id) = build_xfer_request(&kp, "rid123", Operation::Clone, b"GET /x".to_vec()).unwrap();
        let parsed = parse_xfer_envelope(&bytes, Some("rid123")).unwrap();
        assert_eq!(parsed.xfer.operation, Operation::Clone);
        assert_eq!(parsed.env.request_id, req_id);
    }

    #[test]
    fn tampered_request_rejected() {
        let kp = Keypair::generate();
        let (mut bytes, _) = build_xfer_request(&kp, "rid123", Operation::Sync, b"data".to_vec()).unwrap();
        if let Some(last) = bytes.last_mut() {
            *last ^= 0x01;
        }
        assert!(parse_xfer_envelope(&bytes, Some("rid123")).is_err());
    }

    #[test]
    fn wrong_rid_rejected() {
        let kp = Keypair::generate();
        let (bytes, _) = build_xfer_request(&kp, "rid111", Operation::Clone, vec![]).unwrap();
        assert!(parse_xfer_envelope(&bytes, Some("rid222")).is_err());
    }

    #[test]
    fn response_roundtrip() {
        let kp = Keypair::generate();
        let (req_bytes, req_id) = build_xfer_request(&kp, "rid", Operation::Info, vec![]).unwrap();
        let req_env = parse_xfer_envelope(&req_bytes, Some("rid")).unwrap();
        assert_eq!(req_env.xfer.rid, "rid");
        assert_eq!(req_env.xfer.operation, Operation::Info);
        assert_eq!(req_env.env.request_id, req_id);
        let resp_bytes = build_xfer_response(&kp, req_id, "rid", b"hello".to_vec()).unwrap();
        let (_env, resp) = parse_xfer_response(&resp_bytes).unwrap();
        assert!(resp.ok);
        assert_eq!(resp.payload, b"hello".to_vec());
    }

    #[test]
    fn check_peer_auth() {
        let owner = Keypair::generate();
        let allowed = Keypair::generate();
        let stranger = Keypair::generate();

        let vis = crate::identity::Visibility::Private {
            allow: vec![allowed.public_key().to_hex()],
        };
        assert!(check_peer_authorized("", &vis, &allowed.public_key(), Operation::Clone));
        assert!(!check_peer_authorized("", &vis, &stranger.public_key(), Operation::Clone));

        let vis_protected = crate::identity::Visibility::Protected;
        assert!(check_peer_authorized(&owner.public_key().to_did_key(), &vis_protected, &stranger.public_key(), Operation::Clone));
        assert!(!check_peer_authorized(&owner.public_key().to_did_key(), &vis_protected, &stranger.public_key(), Operation::Push));
    }
}