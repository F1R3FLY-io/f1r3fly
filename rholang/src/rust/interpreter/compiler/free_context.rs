use super::exports::SourcePosition;

#[derive(Debug, Clone)]
pub struct FreeContext<T> {
  pub level: usize,
  pub typ: T,
  pub source_position: SourcePosition,
}