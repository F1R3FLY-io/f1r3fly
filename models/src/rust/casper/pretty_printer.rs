// See models/src/main/scala/coop/rchain/casper/PrettyPrinter.scala

use super::protocol::casper_message::CasperMessage;

pub struct PrettyPrinter;

impl PrettyPrinter {
    pub fn build_string_no_limit(b: &[u8]) -> String {
        hex::encode(b)
    }

    pub fn build_string(t: CasperMessage, short: bool) -> String {
        match t {
            CasperMessage::BlockMessage(block_message) => todo!(),
            _ => "Unknown consensus protocol message".to_string(),
        }
    }
}
