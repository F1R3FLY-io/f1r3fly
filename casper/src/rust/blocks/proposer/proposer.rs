// See casper/src/main/scala/coop/rchain/casper/blocks/proposer/Proposer.scala

use models::rust::casper::protocol::casper_message::BlockMessage;

use super::propose_result::ProposeStatus;

pub enum ProposerResult {
    Empty,
    Success(ProposeStatus, BlockMessage),
    Failure(ProposeStatus, i32),
    Started(i32),
}

impl ProposerResult {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn success(status: ProposeStatus, block: BlockMessage) -> Self {
        Self::Success(status, block)
    }

    pub fn failure(status: ProposeStatus, seq_number: i32) -> Self {
        Self::Failure(status, seq_number)
    }

    pub fn started(seq_number: i32) -> Self {
        Self::Started(seq_number)
    }
}
