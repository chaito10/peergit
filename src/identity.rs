use crate::crypto::PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    pub fn from_public_key(pk: &PublicKey) -> Self {
        Self(format!("did:key:{}", pk.to_multibase()))
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private { allow: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIdentity {
    pub rid: String,
    pub name: String,
    pub description: String,
    pub owner_did: String,
    pub visibility: Visibility,
    pub created_at: String,
}

impl RepositoryIdentity {
    pub fn new(
        rid: String,
        name: String,
        description: String,
        owner_did: String,
        visibility: Visibility,
    ) -> Self {
        let now = time::OffsetDateTime::now_utc();
        let created_at = now
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        Self {
            rid,
            name,
            description,
            owner_did,
            visibility,
            created_at,
        }
    }

    pub fn to_json(&self) -> crate::error::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(s: &str) -> crate::error::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Keypair;

    #[test]
    fn did_from_public_key() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let did = Did::from_public_key(&pk);
        assert!(did.0.starts_with("did:key:z"));
    }

    #[test]
    fn repository_identity_serialization() {
        let ri = RepositoryIdentity {
            rid: "abc123".into(),
            name: "test".into(),
            description: "a test repo".into(),
            owner_did: "did:key:z123".into(),
            visibility: Visibility::Public,
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = ri.to_json().unwrap();
        let ri2 = RepositoryIdentity::from_json(&json).unwrap();
        assert_eq!(ri.rid, ri2.rid);
        assert_eq!(ri.name, ri2.name);
    }
}
