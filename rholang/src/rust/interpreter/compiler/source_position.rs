use tree_sitter::Point;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct SourcePosition {
    pub row: usize,
    pub column: usize,
}

impl SourcePosition {
    pub fn new(row: usize, column: usize) -> SourcePosition {
        SourcePosition { row, column }
    }
}

impl Default for SourcePosition {
    fn default() -> Self {
        Self { row: 0, column: 0 }
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.row, self.column)
    }
}

impl From<Point> for SourcePosition {
    fn from(value: Point) -> Self {
        SourcePosition {
            row: value.row,
            column: value.column,
        }
    }
}
