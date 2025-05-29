// See comm/src/main/scala/coop/rchain/comm/discovery/KademliaRPC.scala

use async_trait::async_trait;
use crate::rust::{errors::CommError, peer_node::PeerNode};

#[async_trait]
pub trait KademliaRPC {
    async fn ping(&self, peer: &PeerNode) -> Result<bool, CommError>;
    async fn lookup(&self, key: &[u8], peer: &PeerNode) -> Result<Vec<PeerNode>, CommError>;
}
