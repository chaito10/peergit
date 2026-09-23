use crate::crypto::{Keypair, PublicKey, Signature};
use crate::error::{FossilP2pError, Result};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_PAYLOAD_SIZE: usize = 100 * 1024 * 1024;
pub const AD_NAMESPACE_PREFIX: &str = "/peergit/repos/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    XferRequest = 1,
    XferResponse = 2,
    RepoAdvertisement = 3,
    RepoLookup = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Clone,
    Fetch,
    Push,
    Sync,
    Info,
}

impl Operation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Operation::Clone => "clone",
            Operation::Fetch => "fetch",
            Operation::Push => "push",
            Operation::Sync => "sync",
            Operation::Info => "info",
        }
    }

    pub fn requires_write(&self) -> bool {
        matches!(self, Operation::Push | Operation::Sync)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub version: u16,
    pub message_type: MessageType,
    pub request_id: [u8; 16],
    pub sender: PublicKey,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub signature: Signature,
}

impl Envelope {
    pub fn new(
        message_type: MessageType,
        keypair: &Keypair,
        payload: Vec<u8>,
    ) -> Envelope {
        let mut bytes = payload.clone();
        bytes.extend_from_slice(&crate::crypto::random_bytes(16));
        let hash = crate::crypto::sha256(&bytes);
        let mut req_id = [0u8; 16];
        req_id.copy_from_slice(&hash[..16]);

        Self::with_request_id(message_type, keypair, req_id, payload)
    }

    pub fn with_request_id(
        message_type: MessageType,
        keypair: &Keypair,
        request_id: [u8; 16],
        payload: Vec<u8>,
    ) -> Envelope {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let sender = keypair.public_key();

        let mut envelope = Envelope {
            version: PROTOCOL_VERSION,
            message_type,
            request_id,
            sender,
            timestamp,
            payload,
            signature: Signature([0u8; 64]),
        };
        envelope.signature = envelope.sign(keypair);
        envelope
    }

    pub fn sign(&self, keypair: &Keypair) -> Signature {
        keypair.sign(&self.serialize_signable())
    }

    pub fn verify(&self) -> bool {
        self.sender.verify(&self.serialize_signable(), &self.signature)
    }

    fn serialize_signable(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + 1 + 16 + 32 + 8 + self.payload.len());
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.push(self.message_type as u8);
        bytes.extend_from_slice(&self.request_id);
        bytes.extend_from_slice(&self.sender.to_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn is_fresh(&self, max_age_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.timestamp) <= max_age_secs
    }
}

pub fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XferRequest {
    pub rid: String,
    pub operation: Operation,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XferResponse {
    pub rid: String,
    pub ok: bool,
    pub error: Option<String>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAdvertisement {
    pub rid: String,
    pub owner: PublicKey,
    pub owner_did: String,
    pub creation_nonce: String,
    pub name: String,
    pub description: String,
    pub visibility: String,
    pub protocol_version: u16,
    pub addresses: Vec<String>,
    pub timestamp: u64,
    pub signature: Signature,
}

impl RepoAdvertisement {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rid: &str,
        owner: &Keypair,
        owner_did: &str,
        creation_nonce: &str,
        name: &str,
        description: &str,
        visibility: &str,
        addresses: Vec<String>,
    ) -> RepoAdvertisement {
        let mut ad = RepoAdvertisement {
            rid: rid.to_string(),
            owner: owner.public_key(),
            owner_did: owner_did.to_string(),
            creation_nonce: creation_nonce.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            visibility: visibility.to_string(),
            protocol_version: PROTOCOL_VERSION,
            addresses,
            timestamp: now_unix_secs(),
            signature: Signature([0u8; 64]),
        };
        ad.signature = ad.sign(owner);
        ad
    }

    pub fn sign(&self, keypair: &Keypair) -> Signature {
        keypair.sign(&self.serialize_signable())
    }

    pub fn verify(&self) -> bool {
        self.owner.verify(&self.serialize_signable(), &self.signature)
    }

    fn serialize_signable(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.rid.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&self.owner.to_bytes());
        bytes.extend_from_slice(self.owner_did.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.creation_nonce.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.description.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.visibility.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&self.protocol_version.to_be_bytes());
        for a in &self.addresses {
            bytes.extend_from_slice(a.as_bytes());
            bytes.push(0);
        }
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes
    }

    pub fn is_fresh(&self, max_age_secs: u64) -> bool {
        now_unix_secs().saturating_sub(self.timestamp) <= max_age_secs
    }

    pub fn verify_rid(&self) -> bool {
        let nonce = match hex::decode(&self.creation_nonce) {
            Ok(n) => n,
            Err(_) => return false,
        };
        if nonce.len() != 32 {
            return false;
        }
        let mut nonce_arr = [0u8; 32];
        nonce_arr.copy_from_slice(&nonce);
        crate::repository::FossilRepoManager::verify_rid(&self.rid, &self.owner, &nonce_arr)
    }

    pub fn to_identity(&self) -> crate::identity::RepositoryIdentity {
        crate::identity::RepositoryIdentity {
            rid: self.rid.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            owner_did: self.owner_did.clone(),
            visibility: crate::identity::Visibility::from_db_str(&self.visibility),
            created_at: String::new(),
            protocol_version: self.protocol_version,
            creation_nonce: self.creation_nonce.clone(),
        }
    }
}

pub fn repo_keyspace_key(rid: &str) -> [u8; 32] {
    crate::crypto::sha256(format!("{AD_NAMESPACE_PREFIX}{rid}").as_bytes())
}

pub fn encode_envelope(env: &Envelope) -> Result<Vec<u8>> {
    serde_json::to_vec(env).map_err(FossilP2pError::from)
}

pub fn decode_envelope(bytes: &[u8]) -> Result<Envelope> {
    serde_json::from_slice(bytes).map_err(FossilP2pError::from)
}

pub fn encode_advertisement(ad: &RepoAdvertisement) -> Result<Vec<u8>> {
    serde_json::to_vec(ad).map_err(FossilP2pError::from)
}

pub fn decode_advertisement(bytes: &[u8]) -> Result<RepoAdvertisement> {
    serde_json::from_slice(bytes).map_err(FossilP2pError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_sign_and_verify() {
        let kp = Keypair::generate();
        let env = Envelope::new(MessageType::XferRequest, &kp, b"payload".to_vec());
        assert!(env.verify());
        assert_eq!(env.version, PROTOCOL_VERSION);
    }

    #[test]
    fn envelope_tamper_detected() {
        let kp = Keypair::generate();
        let mut env = Envelope::new(MessageType::XferRequest, &kp, b"payload".to_vec());
        assert!(env.verify());
        env.payload = b"tampered".to_vec();
        assert!(!env.verify());
    }

    #[test]
    fn envelope_json_roundtrip() {
        let kp = Keypair::generate();
        let env = Envelope::new(MessageType::XferResponse, &kp, b"data".to_vec());
        let bytes = encode_envelope(&env).unwrap();
        let decoded = decode_envelope(&bytes).unwrap();
        assert!(decoded.verify());
        assert_eq!(env.request_id, decoded.request_id);
    }

    #[test]
    fn advertisement_sign_and_verify() {
        let kp = Keypair::generate();
        let ad = RepoAdvertisement::new(
            "abc123",
            &kp,
            &kp.public_key().to_did_key(),
            "0000000000000000000000000000000000000000000000000000000000000000",
            "my-repo",
            "a test repo",
            "public",
            vec!["/ip4/127.0.0.1/tcp/4001".into()],
        );
        assert!(ad.verify());
    }

    #[test]
    fn advertisement_tamper_detected() {
        let kp = Keypair::generate();
        let mut ad = RepoAdvertisement::new(
            "abc123",
            &kp,
            &kp.public_key().to_did_key(),
            "00",
            "my-repo",
            "a test repo",
            "public",
            vec![],
        );
        assert!(ad.verify());
        ad.name = "evil-repo".into();
        assert!(!ad.verify());
    }

    #[test]
    fn repo_keyspace_is_stable() {
        let k1 = repo_keyspace_key("rid123");
        let k2 = repo_keyspace_key("rid123");
        let k3 = repo_keyspace_key("rid456");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }
}