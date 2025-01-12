use super::{Assoc, Expr, Fixity, Spanned};

#[derive(Debug)]
pub enum Decl {
    Let {
        ident: String,
        expr: Expr,
    },
    Fn {
        ident: String,
        args: Vec<Spanned<String>>,
        body: Expr,
    },
}

#[derive(Debug, PartialEq)]
pub enum UnparsedDecl {
    Infix {
        ident: Spanned<String>,
        fixity: Fixity,
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
    // Error, // TODO: Error recovery
}

impl std::fmt::Display for UnparsedDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infix { ident, fixity } => {
                let assoc = match fixity.assoc {
                    Assoc::Left => "l",
                    Assoc::Right => "r",
                    Assoc::None => "",
                };
                write!(f, "infix{assoc} {ident} {};", fixity.prec)
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
        }
    }
}
