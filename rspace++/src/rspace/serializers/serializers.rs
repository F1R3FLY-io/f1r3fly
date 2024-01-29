use crate::rspace::internal::{Datum, WaitingContinuation};
use serde::Serialize;

// See rspace/src/main/scala/coop/rchain/rspace/serializers/ScodecSerialize.scala
pub fn encode_datums<A: Clone + Serialize>(datums: &Vec<Datum<A>>) -> Vec<u8> {
    let mut data = bincode::serialize(datums).expect("Serializers: Unable to serialize datums");
    data.sort();
    data
}

pub fn encode_continuations<P: Clone + Serialize, K: Clone + Serialize>(
    conts: &Vec<WaitingContinuation<P, K>>,
) -> Vec<u8> {
    let mut data =
        bincode::serialize(conts).expect("Serializers: Unable to serialize continuations");
    data.sort();
    data
}

pub fn encode_joins<C: Clone + Serialize>(joins: &Vec<Vec<C>>) -> Vec<u8> {
    let mut data = bincode::serialize(joins).expect("Serializers: Unable to serialize joins");
    data.sort();
    data
}

pub fn encode_binary(_data: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut data = bincode::serialize(_data).expect("Serializers: Unable to serialize binary data");
    data.sort();
    data
}
