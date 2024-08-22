// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/Costs.scala
pub struct Cost {
    pub value: i64,
    pub operation: String,
}

impl Cost {
    pub fn create_from_i64(value: i64) -> Cost {
        Cost {
            value,
            operation: "".to_string(),
        }
    }

    pub fn unsafe_max() -> Cost {
        Cost::create_from_i64(i64::MAX)
    }
}
