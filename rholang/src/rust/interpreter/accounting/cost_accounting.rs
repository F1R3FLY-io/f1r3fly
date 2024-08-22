// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/CostAccounting.scala

use std::sync::{Arc, Mutex};

use super::{costs::Cost, has_cost::CostState};

pub struct CostAccounting;

impl CostAccounting {
    fn empty() -> Arc<Mutex<Cost>> {
        Arc::new(Mutex::new(Cost {
            value: 0,
            operation: "init".to_string(),
        }))
    }

    pub fn empty_cost() -> CostState {
        CostState {
            cost: Self::empty(),
            permits: 0,
        }
    }
}
