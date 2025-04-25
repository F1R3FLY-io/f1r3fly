// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/CostAccounting.scala

use super::{CostManager, CostManager, costs::Cost};

pub struct CostAccounting;

impl CostAccounting {
    fn empty() -> Cost {
        Cost {
            value: 0,
            operation: "init".to_string(),
        }
    }

    pub fn empty_cost() -> CostManager {
        CostManager::new(Self::empty(), 1)
    }
}
