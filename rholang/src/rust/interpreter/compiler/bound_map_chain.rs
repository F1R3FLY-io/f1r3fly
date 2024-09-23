use super::exports::*;

#[derive(Debug, Clone)]
pub struct BoundMapChain<T> {
  chain: Vec<BoundMap<T>>,
}

impl<T> BoundMapChain<T> {
  fn new() -> Self {
    BoundMapChain {
      chain: vec![BoundMap::new()],
    }
  }

  fn get(&self, name: &str) -> Option<BoundContext<T>>
  where
    T: Clone,
  {
    self.chain.first().and_then(|map| map.get(name))
  }

  fn find(&self, name: &str) -> Option<(BoundContext<T>, usize)>
  where
    T: Clone,
  {
    self.chain.iter().enumerate().find_map(|(depth, map)| {
      map.get(name).map(|context| (context, depth))
    })
  }

  pub fn put(&mut self, binding: IdContext<T>) {
    if let Some(map) = self.chain.first_mut() {
      map.put(binding);
    }
  }

  fn put_all(&mut self, bindings: Vec<IdContext<T>>) {
    if let Some(map) = self.chain.first_mut() {
      map.put_all(bindings);
    }
  }

  fn absorb_free(&mut self, free_map: FreeMap<T>) {
    if let Some(map) = self.chain.first_mut() {
      map.absorb_free(free_map);
    }
  }

  pub(crate) fn push(&mut self) {
    self.chain.insert(0, BoundMap::new());
  }

  fn count(&self) -> usize {
    self.chain.first().map_or(0, |map| map.count())
  }

  pub(crate) fn depth(&self) -> usize {
    self.chain.len() - 1
  }
}

impl<T> Default for BoundMapChain<T> {
  fn default() -> Self {
    Self::new()
  }
}