use crate::crypto::{Keypair, PublicKey};
use crate::error::Result;
use crate::fossil::FossilCli;
use crate::identity::{Did, RepositoryIdentity, Visibility};
use sha2::Digest;
use std::path::Path;

pub const REPO_IDENTITY_PROTOCOL: u16 = 1;
const RID_DOMAIN_SEPARATOR: &str = "peergit-repository-v1";

pub struct FossilRepoManager {
    pub fossil: FossilCli,
}

impl FossilRepoManager {
    pub fn new(fossil: FossilCli) -> Self {
        Self { fossil }
    }

    pub fn compute_rid(owner_pk: &PublicKey, creation_nonce: &[u8; 32]) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(RID_DOMAIN_SEPARATOR.as_bytes());
        hasher.update(owner_pk.to_bytes());
        hasher.update(creation_nonce);
        let hash = hasher.finalize();
        let rid_bytes: [u8; 32] = hash.into();
        hex::encode(rid_bytes)
    }

    pub fn verify_rid(rid: &str, owner_pk: &PublicKey, creation_nonce: &[u8; 32]) -> bool {
        Self::compute_rid(owner_pk, creation_nonce) == rid
    }

    /// Create a brand new Fossil repository with a PeerGit identity.
    pub fn init_repo(
        &self,
        path: &Path,
        name: &str,
        description: &str,
        owner: &Keypair,
        visibility: Visibility,
    ) -> Result<RepositoryIdentity> {
        let owner_pk = owner.public_key();
        let did = Did::from_public_key(&owner_pk);
        let creation_nonce = random_nonce();
        let rid = Self::compute_rid(&owner_pk, &creation_nonce);

        self.fossil.init(path, name)?;

        let repo_identity = RepositoryIdentity::new(
            rid.clone(),
            name.to_string(),
            description.to_string(),
            did.to_string(),
            visibility,
            hex::encode(creation_nonce),
            REPO_IDENTITY_PROTOCOL,
        );

        write_identity_file(path, &repo_identity)?;

        Ok(repo_identity)
    }

    /// Register an existing Fossil repository as a PeerGit identity without `fossil init`.
    pub fn publish_repo(
        &self,
        path: &Path,
        name: &str,
        description: &str,
        owner: &Keypair,
        visibility: Visibility,
    ) -> Result<RepositoryIdentity> {
        self.fossil.open(path)?;

        let owner_pk = owner.public_key();
        let did = Did::from_public_key(&owner_pk);
        let creation_nonce = random_nonce();
        let rid = Self::compute_rid(&owner_pk, &creation_nonce);

        let repo_identity = RepositoryIdentity::new(
            rid.clone(),
            name.to_string(),
            description.to_string(),
            did.to_string(),
            visibility,
            hex::encode(creation_nonce),
            REPO_IDENTITY_PROTOCOL,
        );

        write_identity_file(path, &repo_identity)?;

        Ok(repo_identity)
    }

    /// Re-register an existing identity file, only updating mutable metadata (name, description, visibility).
    pub fn update_metadata(
        &self,
        path: &Path,
        name: Option<&str>,
        description: Option<&str>,
        visibility: Option<&Visibility>,
    ) -> Result<RepositoryIdentity> {
        let mut identity = Self::open_repo(path)?;
        if let Some(n) = name {
            identity.name = n.to_string();
        }
        if let Some(d) = description {
            identity.description = d.to_string();
        }
        if let Some(v) = visibility {
            identity.visibility = v.clone();
        }
        write_identity_file(path, &identity)?;
        Ok(identity)
    }

    pub fn open_repo(path: &Path) -> Result<RepositoryIdentity> {
        let identity_path = path.join(".fossil-p2p-identity.json");
        if !identity_path.exists() {
            return Err(crate::error::FossilP2pError::Repository(format!(
                "no PeerGit identity found at {}",
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

fn write_identity_file(path: &Path, identity: &RepositoryIdentity) -> Result<()> {
    let identity_path = path.join(".fossil-p2p-identity.json");
    std::fs::write(&identity_path, identity.to_json()?)?;
    Ok(())
}

fn random_nonce() -> [u8; 32] {
    let bytes = crate::crypto::random_bytes(32);
    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&bytes);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rid_is_stable_for_same_nonce() {
        let kp = Keypair::generate();
        let nonce = [7u8; 32];
        let r1 = FossilRepoManager::compute_rid(&kp.public_key(), &nonce);
        let r2 = FossilRepoManager::compute_rid(&kp.public_key(), &nonce);
        assert_eq!(r1, r2);
        assert!(FossilRepoManager::verify_rid(&r1, &kp.public_key(), &nonce));
    }

    #[test]
    fn rid_changes_with_nonce() {
        let kp = Keypair::generate();
        let r1 = FossilRepoManager::compute_rid(&kp.public_key(), &[1u8; 32]);
        let r2 = FossilRepoManager::compute_rid(&kp.public_key(), &[2u8; 32]);
        assert_ne!(r1, r2);
    }

    #[test]
    fn rid_is_independent_of_metadata() {
        let kp = Keypair::generate();
        let nonce = [9u8; 32];
        let rid = FossilRepoManager::compute_rid(&kp.public_key(), &nonce);
        let mut hasher = sha2::Sha256::new();
        hasher.update(RID_DOMAIN_SEPARATOR.as_bytes());
        hasher.update(kp.public_key().to_bytes());
        hasher.update(nonce);
        let hash = hasher.finalize();
        let rid_bytes: [u8; 32] = hash.into();
        assert_eq!(rid, hex::encode(rid_bytes));
    }
}