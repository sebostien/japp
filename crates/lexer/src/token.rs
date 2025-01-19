#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// Concatenation (implicit)
    Concat,
    /// Union '|'
    Union,
    /// Only matches single `char`
    Char(char),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Concat => "".fmt(f),
            Self::Union => "|".fmt(f),
            Self::Char(c) => {
                if matches!(c, '*' | '(' | ')') {
                    write!(f, r"\{c}")
                } else {
                    c.escape_default().fmt(f)
                }
            }
        }
    }
}
