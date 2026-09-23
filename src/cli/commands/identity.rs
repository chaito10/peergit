use crate::error::Result;
use crate::identity::Did;

use super::util::{get_db, get_home, get_keypair};

pub fn cmd_init() -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let did = Did::from_public_key(&pk);

    let db = get_db(&home)?;
    db.store_identity(&pk.to_hex(), &did.to_string(), None)?;

    println!("Node initialized!");
    println!("  Public Key: {}", pk);
    println!("  DID:        {}", did);
    println!("  Peer ID:    {}", pk.to_libp2p_peer_id());
    println!("  Home:       {}", home.path.display());
    Ok(())
}

pub fn cmd_identity() -> Result<()> {
    let home = get_home()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let did = Did::from_public_key(&pk);

    println!("Node Identity:");
    println!("  Public Key: {}", pk);
    println!("  DID:        {}", did);
    println!("  Peer ID:    {}", pk.to_libp2p_peer_id());
    println!("  Key Path:   {}", home.secret_key_path().display());
    Ok(())
}