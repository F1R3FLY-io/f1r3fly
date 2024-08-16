use crypto::rust::public_key::PublicKey;

pub struct BlockData {
    time_stamp: i64,
    block_number: i64,
    sender: PublicKey,
    seq_num: i32,
}
