// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/HasCost.scala

use super::_cost;

pub trait HasCost {
    fn cost(&self) -> &_cost;
}
