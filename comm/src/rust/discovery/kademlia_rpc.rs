// See comm/src/main/scala/coop/rchain/comm/discovery/KademliaRPC.scala

use crate::rust::{errors::CommError, peer_node::PeerNode};

pub trait KademliaRPC {
    fn ping(&self, peer: &PeerNode) -> Result<(), CommError>;
    fn lookup(&self, key: &[u8], peer: &PeerNode) -> Result<Vec<PeerNode>, CommError>;
}
