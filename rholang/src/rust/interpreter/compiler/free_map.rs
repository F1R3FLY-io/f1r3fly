use std::collections::HashMap;
use std::fmt;
use super::exports::*;

#[derive(Debug, Clone)]
pub struct FreeMap<T> {
  pub next_level: usize,
  pub level_bindings: HashMap<String, FreeContext<T>>,
  pub wildcards: Vec<SourcePosition>,
  pub connectives: Vec<(ConnectiveInstance, SourcePosition)>,
}

impl<T> FreeMap<T> {
  pub(crate) fn new() -> Self {
    FreeMap {
      next_level: 0,
      level_bindings: HashMap::new(),
      wildcards: Vec::new(),
      connectives: Vec::new(),
    }
  }

  fn get(&self, name: &str) -> Option<FreeContext<T>>
  where
    T: Clone,
  {
    self.level_bindings.get(name).cloned()
  }

  fn put(&mut self, binding: IdContext<T>) {
    let (name, typ, source_position) = binding;
    self.level_bindings.insert(
      name,
      FreeContext {
        level: self.next_level,
        typ,
        source_position,
      },
    );
    self.next_level += 1;
  }

  fn put_all(&mut self, bindings: Vec<IdContext<T>>) {
    for binding in bindings {
      self.put(binding);
    }
  }

  fn merge(&mut self, free_map: FreeMap<T>) -> Vec<(String, SourcePosition)> {
    let mut shadowed = Vec::new();
    for (name, context) in free_map.level_bindings {
      if self.level_bindings.contains_key(&name) {
        shadowed.push((name.clone(), context.source_position.clone()));
      }
      self.level_bindings.insert(
        name,
        FreeContext {
          level: context.level + self.next_level,
          typ: context.typ,
          source_position: context.source_position,
        },
      );
    }
    self.next_level += free_map.next_level;
    self.wildcards.extend(free_map.wildcards);
    self.connectives.extend(free_map.connectives);
    shadowed
  }

  fn add_wildcard(&mut self, source_position: SourcePosition) {
    self.wildcards.push(source_position);
  }

  fn add_connective(&mut self, connective: ConnectiveInstance, source_position: SourcePosition) {
    self.connectives.push((connective, source_position));
  }

  fn count(&self) -> usize {
    self.next_level + self.wildcards.len() + self.connectives.len()
  }

  fn count_no_wildcards(&self) -> usize {
    self.next_level
  }
}

impl<T> Default for FreeMap<T> {
  fn default() -> Self {
    Self::new()
  }
}
