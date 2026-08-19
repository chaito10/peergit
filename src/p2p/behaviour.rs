use crate::p2p::codec::FossilCodec;
use libp2p::{identify, kad, ping, request_response, swarm::NetworkBehaviour};

#[derive(NetworkBehaviour)]
pub struct FossilP2pBehaviour {
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    pub ping: ping::Behaviour,
    pub xfer: request_response::Behaviour<FossilCodec>,
}

impl FossilP2pBehaviour {
    pub fn new(
        peer_id: libp2p::PeerId,
        key: &libp2p::identity::Keypair,
        kad_protocol: &str,
    ) -> Self {
        let identify = identify::Behaviour::new(identify::Config::new(
            "/peergit/1.0".into(),
            key.public(),
        ));

        let kad_store = kad::store::MemoryStore::new(peer_id);
        let protocol = libp2p::StreamProtocol::try_from_owned(kad_protocol.to_string())
            .expect("valid kademlia protocol starting with /");
        let mut kad = kad::Behaviour::with_config(peer_id, kad_store, kad::Config::new(protocol));
        kad.set_mode(Some(kad::Mode::Server));

        let ping = ping::Behaviour::default();

        let xfer = request_response::Behaviour::new(
            std::iter::once((
                crate::p2p::codec::XFER_PROTOCOL.to_string(),
                request_response::ProtocolSupport::Full,
            )),
            request_response::Config::default()
                .with_request_timeout(std::time::Duration::from_secs(60)),
        );

        Self {
            identify,
            kad,
            ping,
            xfer,
        }
    }
}
