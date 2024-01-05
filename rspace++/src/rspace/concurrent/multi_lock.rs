use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

pub struct MultiLock<K: Ord + Clone> {
    locks: Arc<Mutex<HashMap<K, Arc<Semaphore>>>>,
}

impl<K: Ord + Clone + Hash> MultiLock<K> {
    pub fn new() -> Self {
        MultiLock {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[allow(unused)]
    pub async fn acquire<R, F>(&self, keys: Vec<K>, thunk: F) -> R
    where
        F: FnOnce() -> R,
    {
        let semaphores: Vec<Arc<Semaphore>> = {
            let mut locks = self.locks.lock().unwrap();
            keys.iter()
                .cloned()
                .sorted()
                .map(|key| {
                    let semaphore = locks
                        .entry(key)
                        .or_insert_with(|| Arc::new(Semaphore::new(1)));
                    Arc::clone(semaphore)
                })
                .collect()
        };

        for semaphore in &semaphores {
            let _ = semaphore.acquire().await.unwrap();
        }

        let result = thunk();

        for semaphore in semaphores {
            semaphore.add_permits(1);
        }

        result
    }

    pub fn clean_up(&self) {
        let mut locks = self.locks.lock().unwrap();
        locks.clear();
    }
}
