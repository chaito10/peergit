use crate::error::{FossilP2pError, Result};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Keypair {
    secret: [u8; 32],
    verifying: [u8; 32],
}

impl Keypair {
    pub fn generate() -> Self {
        let mut csprng = rand_core::OsRng;
        let signing = SigningKey::generate(&mut csprng);
        let secret = signing.to_bytes();
        let verifying = signing.verifying_key().to_bytes();
        Self { secret, verifying }
    }

    pub fn from_bytes(secret: &[u8; 32]) -> Result<Self> {
        let signing = SigningKey::from_bytes(secret);
        let verifying = signing.verifying_key().to_bytes();
        Ok(Self {
            secret: *secret,
            verifying,
        })
    }

    fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.secret)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.verifying)
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.signing_key().sign(msg).to_bytes())
    }

    pub fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair> {
        libp2p::identity::Keypair::ed25519_from_bytes(self.secret)
            .map_err(|e| FossilP2pError::Crypto(format!("libp2p key conversion: {e}")))
    }
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut bytes);
    bytes
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash);
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let _vk = VerifyingKey::from_bytes(bytes)
            .map_err(|e| FossilP2pError::Crypto(format!("invalid public key: {e}")))?;
        Ok(Self(*bytes))
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        VerifyingKey::from_bytes(&self.0).map_err(|e| FossilP2pError::Crypto(format!("{e}")))
    }

    pub fn verify(&self, msg: &[u8], sig: &Signature) -> bool {
        if let Ok(vk) = self.verifying_key() {
            let ed_sig = ed25519_dalek::Signature::from_bytes(&sig.0);
            return vk.verify(msg, &ed_sig).is_ok();
        }
        false
    }

    pub fn to_did_key(&self) -> String {
        format!("did:key:{}", self.to_multibase())
    }

    pub fn to_multibase(&self) -> String {
        let mut buf = vec![0xED, 0x01];
        buf.extend_from_slice(&self.0);
        format!("z{}", bs58::encode(&buf).into_string())
    }

    pub fn from_multibase(s: &str) -> Result<Self> {
        let s = s
            .strip_prefix('z')
            .ok_or_else(|| FossilP2pError::Crypto("missing 'z' prefix".into()))?;
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| FossilP2pError::Crypto(format!("bs58 decode: {e}")))?;
        if bytes.len() < 34 || bytes[0] != 0xED || bytes[1] != 0x01 {
            return Err(FossilP2pError::Crypto("invalid multicodec prefix".into()));
        }
        let key_bytes: [u8; 32] = bytes[2..34]
            .try_into()
            .map_err(|_| FossilP2pError::Crypto("invalid key length".into()))?;
        Self::from_bytes(&key_bytes)
    }

    pub fn to_libp2p_peer_id(&self) -> libp2p::PeerId {
        let ed = libp2p::identity::ed25519::PublicKey::try_from_bytes(&self.0)
            .expect("valid ed25519 public key");
        libp2p::identity::PublicKey::from(ed).to_peer_id()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)
            .map_err(|e| FossilP2pError::Crypto(format!("hex decode: {e}")))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| FossilP2pError::Crypto("invalid key length".into()))?;
        Self::from_bytes(&arr)
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_multibase())
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey({})", self.to_multibase())
    }
}

#[derive(Clone, Copy)]
pub struct Signature(pub [u8; 64]);

impl Signature {
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
    }

    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self> {
        Ok(Self(*bytes))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)
            .map_err(|e| FossilP2pError::Crypto(format!("hex decode: {e}")))?;
        let arr: [u8; 64] = bytes
            .try_into()
            .map_err(|_| FossilP2pError::Crypto("invalid signature length".into()))?;
        Self::from_bytes(&arr)
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature({})", self.to_hex())
    }
}

impl Serialize for Signature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Signature::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generate_and_roundtrip() {
        let kp = Keypair::generate();
        let sk = kp.secret_bytes();
        let kp2 = Keypair::from_bytes(&sk).unwrap();
        assert_eq!(kp.public_key().to_bytes(), kp2.public_key().to_bytes());
    }

    #[test]
    fn sign_and_verify() {
        let kp = Keypair::generate();
        let msg = b"hello fossil-p2p";
        let sig = kp.sign(msg);
        assert!(kp.public_key().verify(msg, &sig));
    }

    #[test]
    fn multibase_roundtrip() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let mb = pk.to_multibase();
        let pk2 = PublicKey::from_multibase(&mb).unwrap();
        assert_eq!(pk, pk2);
    }

    #[test]
    fn did_key_format() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let did = pk.to_did_key();
        assert!(did.starts_with("did:key:z"));
    }

    #[test]
    fn libp2p_peer_id_conversion() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let peer_id = pk.to_libp2p_peer_id();
        let keypair_peer_id = kp.to_libp2p_keypair().unwrap().public().to_peer_id();
        assert_eq!(peer_id, keypair_peer_id);
    }

    #[test]
    fn sha256_known_vector() {
        let digest = sha256(b"hello");
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(hex::encode(digest), expected);
    }

    #[test]
    fn random_bytes_len() {
        assert_eq!(random_bytes(16).len(), 16);
        let a = random_bytes(16);
        let b = random_bytes(16);
        assert_ne!(a, b);
    }
}