#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    pub row: usize,
    pub column: usize,
}

impl SourcePosition {
    pub fn new(row: usize, column: usize) -> SourcePosition {
        SourcePosition { row, column }
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.column)
    }
}
