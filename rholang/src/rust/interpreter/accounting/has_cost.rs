use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/HasCost.scala
// _cost = CostState
pub trait HasCost {
    fn cost(&self) -> &CostState;
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/accounting/package.scala
pub struct Cost {
    value: i64,
}

#[derive(Clone)]
pub struct CostState {
    cost: Arc<Mutex<Cost>>,

    semaphore: Arc<Semaphore>,
}

impl CostState {
    pub fn new(initial_value: i64, permits: usize) -> Self {
        CostState {
            cost: Arc::new(Mutex::new(Cost {
                value: initial_value,
            })),

            semaphore: Arc::new(Semaphore::new(permits)),
        }
    }

    pub async fn acquire(&self, _n: usize) {
        let _permit = self.semaphore.acquire().await.unwrap();

        // Permit acquired
    }

    pub async fn get(&self) -> i64 {
        let cost = self.cost.lock().unwrap();

        cost.value
    }

    pub async fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut Cost),
    {
        let mut cost = self.cost.lock().unwrap();

        f(&mut cost);
    }

    pub async fn set(&self, new_cost: Cost) {
        let mut cost = self.cost.lock().unwrap();

        *cost = new_cost;
    }
}
