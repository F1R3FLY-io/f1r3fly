// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/HasCost.scala
// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/package.scala

use std::sync::{Arc, Mutex};

use super::costs::Cost;

// _cost = CostState
pub trait HasCost {
    fn cost(&self) -> &CostState;
}

#[derive(Clone)]
pub struct CostState {
    pub cost: Arc<Mutex<Cost>>,
    pub permits: usize,
}

impl CostState {
    pub fn new(initial_value: i64, permits: usize) -> Self {
        CostState {
            cost: Arc::new(Mutex::new(Cost {
                value: initial_value,
                operation: "".to_string(),
            })),

            permits,
        }
    }

    pub fn acquire(&self, _n: usize) {
        // In a non-async context, we can simply use the permits value directly
        // Here you can implement your own logic for handling permits if needed
        // For now, this function does nothing
    }

    pub fn get(&self) -> i64 {
        let cost = self.cost.lock().unwrap();
        cost.value
    }

    pub fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut Cost),
    {
        let mut cost = self.cost.lock().unwrap();
        f(&mut cost);
    }

    pub fn set(&self, new_cost: Cost) {
        let mut cost = self.cost.lock().unwrap();
        *cost = new_cost;
    }
}
