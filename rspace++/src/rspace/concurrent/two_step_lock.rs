use super::multi_lock::MultiLock;
use std::hash::Hash;
use std::sync::Arc;

pub struct TwoStepLock<K: Ord + Clone> {
    phase_a: Arc<MultiLock<K>>,
    phase_b: Arc<MultiLock<K>>,
}

impl<K: Ord + Clone + Hash> TwoStepLock<K> {
    pub fn new() -> Self {
        TwoStepLock {
            phase_a: Arc::new(MultiLock::new()),
            phase_b: Arc::new(MultiLock::new()),
        }
    }

    #[allow(unused)]
    pub async fn acquire<R, F, G>(&self, keys_a: Vec<K>, phase_two: G, thunk: F) -> R
    where
        F: FnOnce() -> R,
        G: FnOnce() -> Vec<K>,
    {
        let keys_b = self.phase_a.acquire(keys_a, phase_two).await;
        self.phase_b.acquire(keys_b, thunk).await
    }

    pub fn clean_up(&self) {
        self.phase_a.clean_up();
        self.phase_b.clean_up();
    }
}
