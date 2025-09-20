use super::{Expr, Fixity, Ident, Lit};
use japp_util::Spanned;

#[derive(Debug)]
pub enum Decl<'a> {
    Const {
        ident: Ident<'a>,
        rhs: Expr<'a>,
    },
    Fn {
        ident: Ident<'a>,
        type_def: Option<Spanned<Type<'a>>>,
        rows: Vec<FnRow<'a>>,
    },
}

// TODO: Should support: Vec<Vec<X>>, bool, i32, X
// TODO: Dependent, 0, true
#[derive(Debug, PartialEq, Eq)]
pub enum Type<'a> {
    /// Id
    Ident(Ident<'a>),
    Fn {
        /// Type ("->" Type)*
        args: Vec<Spanned<Type<'a>>>,
    },
    Paren {
        /// "(" Type ")"
        inner: Box<Spanned<Type<'a>>>,
    },
    /// Id<Type ("," Type)*>
    Refined {
        ident: Ident<'a>,
        args: Vec<Spanned<Type<'a>>>,
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
            Self::Const { ident, rhs: expr } => {
                write!(f, "const {} = {expr} ;", ident.outer())
            }
            Self::Fn {
                ident,
                type_def,
                rows,
            } => {
                if let Some(type_def) = type_def {
                    // TODO: Print type_def
                    writeln!(f, "{} : {type_def} ;", ident.outer())?;
                }

                let rows = rows
                    .iter()
                    .map(|FnRow { args, body }| {
                        let args = args
                            .iter()
                            .map(Spanned::inner)
                            .map(Lit::to_string)
                            .collect::<Vec<_>>()
                            .join(" ");

                        if args.is_empty() {
                            format!("fn {} = {body} ;", ident.outer())
                        } else {
                            format!("fn {} {args} = {body} ;", ident.outer())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                write!(f, "{rows}")
            }
        }
    }
}

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Ident(i) => i.fmt(f),
            Type::Fn { args } => {
                write!(
                    f,
                    "{}",
                    args.iter()
                        .map(|arg| { arg.to_string() })
                        .collect::<Vec<_>>()
                        .join(" -> ")
                )
            }
            Type::Refined { ident, args } => write!(
                f,
                "{}<{}>",
                ident.outer(),
                args.iter()
                    .map(|arg| { arg.to_string() })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Paren { inner } => {
                write!(f, "( {inner} )")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnparsedDecl<'a> {
    Infix {
        ident: Ident<'a>,
        fixity: Fixity,
    },
    Const {
        ident: Ident<'a>,
        rhs: nom_span::Spanned<&'a str>,
    },
    Fn {
        ident: Ident<'a>,
        args: Vec<Spanned<Lit<'a>>>,
        body: nom_span::Spanned<&'a str>,
    },
    FnSig {
        ident: Ident<'a>,
        sig: Spanned<Type<'a>>,
    }, // Error, // TODO: Error recovery
}
