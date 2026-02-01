use parser::Ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit<'a> {
    Null,
    Bool(bool),
    Int(isize),
    Ident(Ident<'a>),
}

impl std::fmt::Display for Lit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => "null".fmt(f),
            Self::Bool(b) => b.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Ident(i) => i.fmt(f),
        }
    }
}
