// See models/src/main/scala/coop/rchain/models/StringSyntax.scala
pub struct StringOps;

impl StringOps {
    pub fn decode_hex(s: String) -> Option<Vec<u8>> {
        hex::decode(&s).ok()
    }

    pub fn unsafe_decode_hex(s: String) -> Vec<u8> {
        hex::decode(&s).unwrap()
    }
}
