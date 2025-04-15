// See casper/src/main/scala/coop/rchain/casper/util/ProtoUtil.scala

use block_storage::rust::{
    dag::block_dag_key_value_storage::KeyValueDagRepresentation,
    key_value_block_store::KeyValueBlockStore,
};
use crypto::rust::hash::blake2b256::Blake2b256;
use models::rust::{
    block_hash::BlockHash,
    block_metadata::BlockMetadata,
    casper::protocol::casper_message::{BlockMessage, Body, Header, Justification},
};
use shared::rust::{store::key_value_store::KvStoreError, ByteString};

pub fn parent_hashes(block: &BlockMessage) -> Vec<prost::bytes::Bytes> {
    block
        .header
        .parents_hash_list
        .iter()
        .map(|bytes| bytes.clone())
        .collect()
}

pub fn get_parents(
    block_store: &mut KeyValueBlockStore,
    block: &BlockMessage,
) -> Vec<BlockMessage> {
    parent_hashes(block)
        .into_iter()
        .map(|bytes| block_store.get_unsafe(bytes))
        .collect()
}

pub fn get_parents_metadata(
    dag: &mut KeyValueDagRepresentation,
    block: &BlockMetadata,
) -> Result<Vec<BlockMetadata>, KvStoreError> {
    block
        .parents
        .iter()
        .map(|parent| dag.lookup_unsafe(parent))
        .collect()
}

pub fn block_header(parent_hashes: Vec<ByteString>, version: i64, timestamp: i64) -> Header {
    Header {
        parents_hash_list: parent_hashes
            .into_iter()
            .map(|bytes| bytes.into())
            .collect(),
        timestamp,
        version,
        extra_bytes: prost::bytes::Bytes::new(),
    }
}

pub fn unsigned_block_proto(
    body: Body,
    header: Header,
    justifications: Vec<Justification>,
    shard_id: String,
    seq_num: Option<i32>,
) -> BlockMessage {
    let seq_num = seq_num.unwrap_or(0);
    let mut block = BlockMessage {
        block_hash: prost::bytes::Bytes::new(),
        header,
        body,
        justifications,
        sender: prost::bytes::Bytes::new(),
        seq_num,
        sig: prost::bytes::Bytes::new(),
        sig_algorithm: "".to_string(),
        shard_id,
        extra_bytes: prost::bytes::Bytes::new(),
    };

    let hash = hash_block(&block);
    block.block_hash = hash.into();
    block
}

pub fn hash_block(block: &BlockMessage) -> BlockHash {
    use prost::Message;

    let bytes: Vec<u8> = block
        .header
        .to_proto()
        .encode_to_vec()
        .into_iter()
        .chain(block.body.to_proto().encode_to_vec().into_iter())
        .chain(block.sender.clone().into_iter())
        .chain(block.sig_algorithm.as_bytes().to_vec().into_iter())
        .chain(block.seq_num.to_le_bytes().into_iter())
        .chain(block.shard_id.as_bytes().to_vec().into_iter())
        .chain(block.extra_bytes.clone().into_iter())
        .collect();

    Blake2b256::hash(bytes).into()
}
