use super::exports::SourcePosition;

// Define the IdContext type alias
pub type IdContext<T> = (String, T, SourcePosition);