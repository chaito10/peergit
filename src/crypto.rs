use crate::error::{FossilP2pError, Result};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Keypair {
    signing: SigningKey,
    verifying: [u8; 32],
}

impl Keypair {
    pub fn generate() -> Self {
        let mut csprng = rand_core::OsRng;
        let signing = SigningKey::generate(&mut csprng);
        let verifying = signing.verifying_key().to_bytes();
        Self { signing, verifying }
    }

    pub fn from_bytes(secret: &[u8; 32]) -> Result<Self> {
        let signing = SigningKey::from_bytes(secret);
        let verifying = signing.verifying_key().to_bytes();
        Ok(Self { signing, verifying })
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.verifying)
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.signing.sign(msg).to_bytes())
    }

    pub fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair> {
        libp2p::identity::Keypair::ed25519_from_bytes(self.signing.to_bytes())
            .map_err(|e| FossilP2pError::Crypto(format!("libp2p key conversion: {e}")))
    }
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
        VerifyingKey::from_bytes(&self.0)
            .map_err(|e| FossilP2pError::Crypto(format!("{e}")))
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
        let keypair = libp2p::identity::Keypair::ed25519_from_bytes(self.0)
            .expect("valid ed25519 key");
        keypair.public().to_peer_id()
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
        let _peer_id = pk.to_libp2p_peer_id();
    }
}
