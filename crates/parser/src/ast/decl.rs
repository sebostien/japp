use super::{Assoc, Expr, Fixity, Lit};
use japp_util::Spanned;

#[derive(Debug)]
pub enum Decl<'a> {
    Let {
        ident: &'a str,
        rhs: Expr<'a>,
    },
    Fn {
        ident: &'a str,
        rows: Vec<FnRow<'a>>,
    },
}

#[derive(Debug)]
pub struct FnRow<'a> {
    pub args: Vec<Spanned<Lit<'a>>>,
    pub body: Expr<'a>,
}

impl std::fmt::Display for Decl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Let { ident, rhs: expr } => {
                write!(f, "let {ident} = {expr} ;")
            }
            Self::Fn { ident, rows } => {
                for FnRow { args, body } in rows {
                    let args = args
                        .iter()
                        .map(Spanned::inner)
                        .map(Lit::to_string)
                        .collect::<Vec<_>>()
                        .join(" ");

                    write!(f, "fn {ident} {args} = {body} ;")?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UnparsedDecl<'a> {
    Infix {
        ident: nom_span::Spanned<&'a str>,
        fixity: Fixity,
    },
    Let {
        ident: nom_span::Spanned<&'a str>,
        rhs: nom_span::Spanned<&'a str>,
    },
    Fn {
        ident: nom_span::Spanned<&'a str>,
        args: Vec<nom_span::Spanned<&'a str>>,
        body: nom_span::Spanned<&'a str>,
    },
    // Error, // TODO: Error recovery
}

impl std::fmt::Display for UnparsedDecl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infix { ident, fixity } => {
                let assoc = match fixity.assoc {
                    Assoc::Left => "l",
                    Assoc::Right => "r",
                    Assoc::None => "",
                };
                write!(f, "infix{assoc} {} {};", ident.data(), fixity.prec)
            }
            Self::Let { ident, rhs } => {
                write!(f, "let {} = {} ;", ident.data(), rhs.data())
            }
            Self::Fn { ident, args, body } => {
                let args = args
                    .iter()
                    .map(nom_span::Spanned::data)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "fn {} {args} = {} ;", ident.data(), body.data())
            }
        }
    }
}
