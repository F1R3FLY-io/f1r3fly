// See casper/src/test/scala/coop/rchain/casper/engine/Setup.scala

use comm::rust::peer_node::{Endpoint, NodeIdentifier, PeerNode};
use prost::bytes::Bytes;

fn endpoint(port: u32) -> Endpoint {
    Endpoint {
        host: "host".to_string(),
        tcp_port: port,
        udp_port: port,
    }
}

pub fn peer_node(name: &str, port: u32) -> PeerNode {
    PeerNode {
        id: NodeIdentifier {
            key: Bytes::from(name.as_bytes().to_vec()),
        },
        endpoint: endpoint(port),
    }
}
