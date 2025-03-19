pub mod shared;
pub mod store;

pub type ByteVector = Vec<u8>;
pub type ByteBuffer = Vec<u8>;
pub type Byte = u8;
pub type ByteString = Vec<u8>;
pub type BitSet = Vec<u8>;
pub type BitVector = Vec<u8>;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BytesWrapper(pub prost::bytes::Bytes);

impl BytesWrapper {
    pub fn into_bytes(&self) -> prost::bytes::Bytes {
        self.0.clone()
    }
}

impl From<prost::bytes::Bytes> for BytesWrapper {
    fn from(bytes: prost::bytes::Bytes) -> Self {
        BytesWrapper(bytes)
    }
}

impl From<Vec<u8>> for BytesWrapper {
    fn from(bytes: Vec<u8>) -> Self {
        BytesWrapper(prost::bytes::Bytes::from(bytes))
    }
}

impl Serialize for BytesWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for BytesWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        Ok(BytesWrapper(prost::bytes::Bytes::from(bytes)))
    }
}

impl proptest::arbitrary::Arbitrary for BytesWrapper {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;

        proptest::collection::vec(any::<u8>(), 32)
            .prop_map(|v| BytesWrapper(prost::bytes::Bytes::from(v)))
            .boxed()
    }
}
