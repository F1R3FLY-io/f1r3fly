use super::exports::SourcePosition;

#[derive(Debug, Clone)]
pub struct FreeContext<T> {
  pub level: usize,
  pub typ: T,
  pub source_position: SourcePosition,
}

// impl<T> Clone for FreeContext<T> {
//   fn clone(&self) -> Self {
//     FreeContext {
//       level: self.level,
//       typ: self.typ.clone(),
//       source_position: self.source_position.clone(),
//     }
//   }
// }