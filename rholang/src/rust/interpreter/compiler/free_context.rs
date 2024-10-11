use super::exports::SourcePosition;

#[derive(Debug, Clone, PartialEq)]
pub struct FreeContext<T: Clone> {
  pub level: usize,
  pub typ: T,
  pub source_position: SourcePosition,
}