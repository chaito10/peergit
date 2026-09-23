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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private {
        #[serde(default)]
        allow: Vec<String>,
    },
    Protected,
}

impl Visibility {
    pub fn as_db_str(&self) -> String {
        match self {
            Visibility::Public => "public".to_string(),
            Visibility::Private { .. } => "private".to_string(),
            Visibility::Protected => "protected".to_string(),
        }
    }

    pub fn from_db_str(s: &str) -> Visibility {
        match s {
            "private" => Visibility::Private { allow: vec![] },
            "protected" => Visibility::Protected,
            _ => Visibility::Public,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIdentity {
    pub rid: String,
    pub name: String,
    pub description: String,
    pub owner_did: String,
    pub visibility: Visibility,
    pub created_at: String,
    pub protocol_version: u16,
    pub creation_nonce: String,
}

impl RepositoryIdentity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rid: String,
        name: String,
        description: String,
        owner_did: String,
        visibility: Visibility,
        creation_nonce: String,
        protocol_version: u16,
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
            protocol_version,
            creation_nonce,
        }
    }

    pub fn owner_authorizes(&self, peer_key: &PublicKey) -> bool {
        match &self.visibility {
            Visibility::Public => true,
            Visibility::Private { allow } => {
                allow.iter().any(|k| k == &peer_key.to_hex())
                    || allow.iter().any(|k| k == &peer_key.to_multibase())
            }
            Visibility::Protected => true,
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
            protocol_version: 1,
            creation_nonce: "deadbeef".into(),
        };
        let json = ri.to_json().unwrap();
        let ri2 = RepositoryIdentity::from_json(&json).unwrap();
        assert_eq!(ri.rid, ri2.rid);
        assert_eq!(ri.name, ri2.name);
    }

    #[test]
    fn visibility_db_roundtrip() {
        assert_eq!(Visibility::from_db_str("public"), Visibility::Public);
        assert_eq!(Visibility::from_db_str(&Visibility::Public.as_db_str()), Visibility::Public);
        assert_eq!(Visibility::from_db_str(&Visibility::Protected.as_db_str()), Visibility::Protected);
        let priv_v = Visibility::Private { allow: vec![] };
        assert_eq!(Visibility::from_db_str(&priv_v.as_db_str()), Visibility::Private { allow: vec![] });
    }

    #[test]
    fn private_visibility_authorizes_only_allowlist() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let other = Keypair::generate();
        let other_pk = other.public_key();

        let ri = RepositoryIdentity {
            rid: "abc".into(),
            name: "test".into(),
            description: String::new(),
            owner_did: "did:key:z1".into(),
            visibility: Visibility::Private {
                allow: vec![pk.to_hex()],
            },
            created_at: "2026-01-01T00:00:00Z".into(),
            protocol_version: 1,
            creation_nonce: "nonce".into(),
        };
        assert!(ri.owner_authorizes(&pk));
        assert!(!ri.owner_authorizes(&other_pk));
    }
}