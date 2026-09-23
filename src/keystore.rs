use crate::crypto::Keypair;
use crate::error::{FossilP2pError, Result};
use crate::home::Home;
use std::fs;

/// Load the node identity keypair from disk.
/// Migrates the legacy `keys/fossil-p2p` path to the new `keys/identity.key` path.
pub fn load_keypair(home: &Home) -> Result<Keypair> {
    let primary = home.secret_key_path();
    let legacy = home.keys().join("fossil-p2p");

    let path = if primary.exists() {
        primary
    } else if legacy.exists() {
        let content = fs::read_to_string(&legacy)?;
        home.write_secret_key(content.trim())?;
        let _ = fs::remove_file(&legacy);
        home.secret_key_path()
    } else {
        return Err(FossilP2pError::Identity(format!(
            "no identity key found at {}. Run 'peergit init' first.",
            primary.display()
        )));
    };

    let sk_hex = fs::read_to_string(&path)?;
    let sk_bytes = hex::decode(sk_hex.trim())
        .map_err(|e| FossilP2pError::Crypto(format!("invalid key hex: {e}")))?;
    let sk_arr: [u8; 32] = sk_bytes
        .try_into()
        .map_err(|_| FossilP2pError::Crypto("invalid secret key length".into()))?;
    Keypair::from_bytes(&sk_arr)
}

/// Load the identity keypair, generating a fresh one if none exists.
pub fn load_or_create_keypair(home: &Home) -> Result<Keypair> {
    match load_keypair(home) {
        Ok(kp) => Ok(kp),
        Err(_) => {
            home.init()?;
            let keypair = Keypair::generate();
            home.write_secret_key(&hex::encode(keypair.secret_bytes()))?;
            let _ = fs::write(
                home.public_key_path(),
                hex::encode(keypair.public_key().to_bytes()),
            );
            Ok(keypair)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::Home;

    #[test]
    fn create_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::from_path(dir.path().to_path_buf()).unwrap();
        let kp1 = load_or_create_keypair(&home).unwrap();
        let kp2 = load_keypair(&home).unwrap();
        assert_eq!(kp1.public_key().to_bytes(), kp2.public_key().to_bytes());
    }

    #[test]
    fn missing_key_errors() {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::from_path(dir.path().to_path_buf()).unwrap();
        assert!(load_keypair(&home).is_err());
    }

    #[test]
    fn legacy_key_migration() {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::from_path(dir.path().to_path_buf()).unwrap();
        home.init().unwrap();
        let kp = Keypair::generate();
        fs::write(home.keys().join("fossil-p2p"), hex::encode(kp.secret_bytes())).unwrap();

        let migrated = load_keypair(&home).unwrap();
        assert_eq!(kp.public_key().to_bytes(), migrated.public_key().to_bytes());
        assert!(home.secret_key_path().exists());
        assert!(!home.keys().join("fossil-p2p").exists());
    }
}