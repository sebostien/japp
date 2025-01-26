use nom_span::Spanned;

use super::{expr::EvalError, Ident};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit<'a> {
    Bool(bool),
    Int(isize),
    Ident(Ident<'a>),
}

impl std::fmt::Display for Lit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => b.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Ident(i) => i.fmt(f),
        }
    }
}

impl<'source> From<Spanned<&'source str>> for Lit<'source> {
    fn from(spanned: Spanned<&'source str>) -> Self {
        let s = *spanned.data();
        match s {
            "true" => Lit::Bool(true),
            "false" => Lit::Bool(false),
            _ => {
                if let Ok(n) = s.parse() {
                    Lit::Int(n)
                } else {
                    Lit::Ident(Ident::new(spanned.byte_offset()..s.len(), s))
                }
            }
        }
    }
}

impl<'source> From<Ident<'source>> for Lit<'source> {
    fn from(ident: Ident<'source>) -> Self {
        let s = ident.outer();
        match s {
            "true" => Lit::Bool(true),
            "false" => Lit::Bool(false),
            _ => {
                if let Ok(n) = s.parse() {
                    Lit::Int(n)
                } else {
                    Lit::Ident(ident)
                }
            }
        }
    }
}

impl<'source> From<japp_util::Spanned<&'source str>> for Lit<'source> {
    fn from(spanned: japp_util::Spanned<&'source str>) -> Self {
        let s = spanned.inner;
        match s {
            "true" => Lit::Bool(true),
            "false" => Lit::Bool(false),
            _ => {
                if let Ok(n) = s.parse() {
                    Lit::Int(n)
                } else {
                    Lit::Ident(Ident::new(spanned.span, spanned.inner))
                }
            }
        }
    }
}

impl<'source> TryFrom<Lit<'source>> for isize {
    type Error = EvalError<'source>;

    fn try_from(value: Lit<'source>) -> Result<Self, Self::Error> {
        if let Lit::Int(n) = value {
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
