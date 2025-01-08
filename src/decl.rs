use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct Spanned<T> {
    pub span: Range<usize>,
    pub inner: T,
}

#[derive(Debug, PartialEq)]
pub enum Decl {
    Infix {
        ident: Spanned<String>,
        prec: usize,
        assoc: Assoc,
    },
    Let {
        ident: Spanned<String>,
        rhs: Spanned<String>,
    },
    Fn {
        ident: Spanned<String>,
        args: Vec<Spanned<String>>,
        body: Spanned<String>,
    },
    Error,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Assoc {
    /// Left associativity
    ///
    /// `1 / 2 / 3 = (1 / 2) / 3`
    Left,
    /// Right associativity
    ///
    /// `1^2^3 = 1^(2^3)`
    Right,
    /// No associativity
    ///
    /// Ok : "1 == (2 == 3)"
    /// Err: "1 == 2 == 3"
    None,
}

impl std::fmt::Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infix { ident, prec, assoc } => {
                let assoc = match assoc {
                    Assoc::Left => "l",
                    Assoc::Right => "r",
                    Assoc::None => "",
                };
                write!(f, "infix{assoc} {ident} {prec};")
            }
            Self::Let { ident, rhs } => {
                write!(f, "let {ident} = {rhs} ;")
            }
            Self::Fn { ident, args, body } => {
                let args = args
                    .iter()
                    .map(Spanned::to_string)
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "fn {ident} {args} = {body} ;")
            }
            Self::Error => Ok(()),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
