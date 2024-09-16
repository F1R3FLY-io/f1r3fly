#[derive(Debug, Clone)]
pub struct SourcePosition {
  pub row: usize,
  pub column: usize,
}

impl std::fmt::Display for SourcePosition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.row, self.column)
  }
}