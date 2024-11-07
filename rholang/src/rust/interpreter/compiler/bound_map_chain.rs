use super::exports::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BoundMapChain<T> {
    pub(crate) chain: Vec<BoundMap<T>>,
}

impl<T: Clone> BoundMapChain<T> {
    pub fn new() -> Self {
        BoundMapChain {
            chain: vec![BoundMap::new()],
        }
    }

    pub fn get(&self, name: &str) -> Option<BoundContext<T>> {
        self.chain.first().and_then(|map| map.get(name))
    }

    pub fn find(&self, name: &str) -> Option<(BoundContext<T>, usize)> {
        self.chain
            .iter()
            .enumerate()
            .find_map(|(depth, map)| map.get(name).map(|context| (context, depth)))
    }

    pub fn put(&self, binding: IdContext<T>) -> BoundMapChain<T> {
        let mut new_chain = self.chain.clone();
        if let Some(map) = new_chain.first_mut() {
            new_chain[0] = map.put(binding);
        }
        BoundMapChain { chain: new_chain }
    }

    pub fn put_all(&self, bindings: Vec<IdContext<T>>) -> BoundMapChain<T> {
        let mut new_chain = self.chain.clone();
        if let Some(map) = new_chain.first_mut() {
            map.put_all(bindings);
        }
        BoundMapChain { chain: new_chain }
    }

    pub(crate) fn absorb_free(&self, free_map: FreeMap<T>) -> BoundMapChain<T> {
        let mut new_chain = self.chain.clone();
        if let Some(map) = new_chain.first_mut() {
            new_chain[0] = map.absorb_free(free_map);
        }
        BoundMapChain { chain: new_chain }
    }

    pub fn push(&self) -> BoundMapChain<T> {
        let mut new_chain = self.chain.clone();
        new_chain.insert(0, BoundMap::new());
        BoundMapChain { chain: new_chain }
    }

    pub fn get_count(&self) -> usize {
        self.chain.first().map_or(0, |map| map.get_count())
    }

    pub(crate) fn depth(&self) -> usize {
        self.chain.len() - 1
    }
}

impl<T: Clone> Default for BoundMapChain<T> {
    fn default() -> Self {
        Self::new()
    }
}
