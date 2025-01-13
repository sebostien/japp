use super::expr::EvalError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit<'a> {
    Bool(bool),
    Num(isize),
    Ident(&'a str),
}

impl std::fmt::Display for Lit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => b.fmt(f),
            Self::Num(i) => i.fmt(f),
            Self::Ident(i) => i.fmt(f),
        }
    }
}

impl<'source> From<&'source str> for Lit<'source> {
    fn from(s: &'source str) -> Self {
        if s == "true" {
            Lit::Bool(true)
        } else if s == "false" {
            Lit::Bool(false)
        } else if let Ok(num) = s.parse() {
            Lit::Num(num)
        } else {
            Lit::Ident(s)
        }
    }
}

impl<'source> TryFrom<Lit<'source>> for isize {
    type Error = EvalError<'source>;

    fn try_from(value: Lit<'source>) -> Result<Self, Self::Error> {
        if let Lit::Num(n) = value {
            Ok(n)
        } else {
            Err(EvalError::ExpectedInt(value))
        }
    }
}

impl<'source> TryFrom<Lit<'source>> for bool {
    type Error = EvalError<'source>;

    fn try_from(value: Lit<'source>) -> Result<Self, Self::Error> {
        if let Lit::Bool(b) = value {
            Ok(b)
        } else {
            Err(EvalError::ExpectedBool(value))
        }
    }
}
