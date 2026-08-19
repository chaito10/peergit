use crate::error::Result;
use crate::fossil::FossilCli;
use crate::identity::{Did, RepositoryIdentity, Visibility};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct FossilRepoManager {
    pub fossil: FossilCli,
}

impl FossilRepoManager {
    pub fn new(fossil: FossilCli) -> Self {
        Self { fossil }
    }

    pub fn compute_rid(name: &str, description: &str, owner_did: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(description.as_bytes());
        hasher.update(owner_did.as_bytes());
        let hash = hasher.finalize();
        let rid_bytes: [u8; 32] = hash.into();
        hex::encode(rid_bytes)
    }

    pub fn init_repo(
        &self,
        path: &Path,
        name: &str,
        description: &str,
        owner_pk: &crate::crypto::PublicKey,
    ) -> Result<RepositoryIdentity> {
        let did = Did::from_public_key(owner_pk);
        let rid = Self::compute_rid(name, description, &did.to_string());

        self.fossil.init(path, name)?;

        let repo_identity = RepositoryIdentity::new(
            rid.clone(),
            name.to_string(),
            description.to_string(),
            did.to_string(),
            Visibility::Public,
        );

        let identity_path = path.join(".fossil-p2p-identity.json");
        std::fs::write(&identity_path, repo_identity.to_json()?)?;

        Ok(repo_identity)
    }

    pub fn open_repo(path: &Path) -> Result<RepositoryIdentity> {
        let identity_path = path.join(".fossil-p2p-identity.json");
        if !identity_path.exists() {
            return Err(crate::error::FossilP2pError::Repository(format!(
                "no Fossil-P2P identity found at {}",
                path.display()
            )));
        }
        let content = std::fs::read_to_string(&identity_path)?;
        RepositoryIdentity::from_json(&content)
    }

    pub fn status(&self, path: &Path) -> Result<String> {
        self.fossil.status(path)
    }

    pub fn add(&self, path: &Path, paths: &[&str]) -> Result<()> {
        self.fossil.add(path, paths)
    }

    pub fn commit(&self, path: &Path, message: &str) -> Result<String> {
        self.fossil.commit(path, message, true)
    }

    pub fn timeline(&self, path: &Path, count: Option<usize>) -> Result<String> {
        self.fossil.timeline(path, count)
    }

    pub fn branches(&self, path: &Path) -> Result<String> {
        self.fossil.branches(path)
    }
}
