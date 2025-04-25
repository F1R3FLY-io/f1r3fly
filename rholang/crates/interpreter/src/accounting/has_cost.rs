// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/HasCost.scala

use super::CostManager;

pub trait HasCost {
    fn cost(&self) -> &CostManager;
}
