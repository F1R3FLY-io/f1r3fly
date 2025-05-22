// See comm/src/main/scala/coop/rchain/comm/transport/TransportLayer.scala
// See comm/src/main/scala/coop/rchain/comm/transport/TransportLayerSyntax.scala
// See casper/src/main/scala/coop/rchain/casper/util/comm/CommUtil.scala

use std::time::Duration;

use models::{
    routing::{Packet, Protocol},
    rust::{block_hash::BlockHash, casper::protocol::packet_type_tag::ToPacket},
};

use crate::rust::{
    errors::CommError,
    peer_node::PeerNode,
    rp::{protocol_helper, rp_conf::RPConf},
};

pub struct Blob {
    pub sender: PeerNode,
    pub packet: Packet,
}

pub trait TransportLayer {
    fn send(peer: &PeerNode, msg: &Protocol) -> Result<(), CommError>;

    fn broadcast(peers: &[PeerNode], msg: &Protocol) -> Result<(), CommError>;

    fn stream(peer: &PeerNode, blob: &Blob) -> Result<(), CommError>;

    fn stream_mult(peers: &[PeerNode], blob: &Blob) -> Result<(), CommError>;

    // See comm/src/main/scala/coop/rchain/comm/transport/TransportLayerSyntax.scala

    fn send_packet_to_peer(conf: &RPConf, peer: &PeerNode, msg: Packet) -> Result<(), CommError> {
        let protocol_msg = protocol_helper::packet(&conf.local, &conf.network_id, msg);
        Self::send(peer, &protocol_msg)
    }

    fn send_message_to_peer(
        conf: &RPConf,
        peer: &PeerNode,
        msg: &impl ToPacket,
    ) -> Result<(), CommError> {
        let packet = msg.mk_packet();
        Self::send_packet_to_peer(conf, peer, packet)
    }

    fn stream_packet_to_peer(
        conf: RPConf,
        peer: &PeerNode,
        packet: Packet,
    ) -> Result<(), CommError> {
        let blob = Blob {
            sender: conf.local,
            packet,
        };
        Self::stream(peer, &blob)
    }

    fn stream_message_to_peer(
        conf: RPConf,
        peer: &PeerNode,
        msg: &impl ToPacket,
    ) -> Result<(), CommError> {
        let packet = msg.mk_packet();
        Self::stream_packet_to_peer(conf, peer, packet)
    }

    fn send_to_bootstrap(conf: &RPConf, msg: &impl ToPacket) -> Result<(), CommError> {
        let bootstrap = conf
            .bootstrap
            .as_ref()
            .ok_or_else(|| CommError::ConfigError("No bootstrap peer configured".to_string()))?;
        Self::send_message_to_peer(conf, bootstrap, msg)
    }

    // See casper/src/main/scala/coop/rchain/casper/util/comm/CommUtil.scala

    fn send_packet_to_peers(
        conf: &RPConf,
        message: Packet,
        scope_size: Option<usize>,
    ) -> Result<(), CommError> {
        let max = scope_size.unwrap_or(conf.max_num_of_connections);

        todo!()
    }

    fn stream_packet_to_peers(
        conf: &RPConf,
        message: Packet,
        scope_size: Option<usize>,
    ) -> Result<(), CommError> {
        todo!()
    }

    fn send_with_retry(
        conf: &RPConf,
        message: Packet,
        peer: &PeerNode,
        retry_after: Duration,
    ) -> Result<(), CommError> {
        todo!()
    }

    fn request_for_block(conf: &RPConf, peer: &PeerNode, hash: BlockHash) -> Result<(), CommError> {
        todo!()
    }

    fn send_message_to_peers(
        conf: &RPConf,
        message: &impl ToPacket,
        scope_size: Option<usize>,
    ) -> Result<(), CommError> {
        todo!()
    }

    fn stream_message_to_peers(
        conf: &RPConf,
        message: &impl ToPacket,
        scope_size: Option<usize>,
    ) -> Result<(), CommError> {
        todo!()
    }

    fn send_block_hash(conf: &RPConf, peer: &PeerNode, hash: BlockHash) -> Result<(), CommError> {
        todo!()
    }

    fn broadcast_has_block_request(conf: &RPConf, hash: BlockHash) -> Result<(), CommError> {
        todo!()
    }

    fn broadcast_request_for_block(conf: &RPConf, hash: BlockHash) -> Result<(), CommError> {
        todo!()
    }

    fn send_fork_choice_tip_request(conf: &RPConf) -> Result<(), CommError> {
        todo!()
    }

    fn request_approved_block(conf: &RPConf, trim_state: bool) -> Result<(), CommError> {
        todo!()
    }
}
