use super::exports::SourcePosition;

#[derive(Debug, Clone)]
pub struct BoundContext<T> {
  pub index: usize,
  pub typ: T,
  pub source_position: SourcePosition,
}