use super::{Expr, Fixity, Ident};
use japp_util::Spanned;
use spressions::{Spression, ToSpression};

#[derive(Debug)]
pub enum Decl<'a> {
    Const {
        ident: Ident<'a>,
        rhs: Expr<'a>,
    },
    Fn {
        ident: Ident<'a>,
        type_def: Option<Spanned<Type<'a>>>,
        args: Vec<Ident<'a>>,
        body: Expr<'a>,
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

impl std::fmt::Display for Decl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const { ident, rhs: expr } => {
                write!(f, "const {} = {expr} ;", ident.outer())
            }
            Self::Fn {
                ident,
                type_def,
                args,
                body,
            } => {
                if let Some(type_def) = type_def {
                    writeln!(f, "{} : {type_def} ;", ident.outer())?;
                }

                let args = args
                    .iter()
                    .map(|arg| arg.inner().to_string() + " ")
                    .collect::<String>();

                write!(f, "fn {} {args}= {body} ;", ident.outer)
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
        rhs: Spanned<&'a str>,
    },
    Fn {
        ident: Ident<'a>,
        args: Vec<Ident<'a>>,
        body: Spanned<&'a str>,
    },
    FnSig {
        ident: Ident<'a>,
        sig: Spanned<Type<'a>>,
    }, // Error, // TODO: Error recovery
}

impl<'a> ToSpression for Decl<'a> {
    fn to_spression(self) -> Spression {
        match self {
            Decl::Const { ident, rhs } => Spression {
                node: "Const".to_string(),
                span: None,
                data: vec![format!("\"{}\"", ident.to_string())],
                children: vec![rhs.to_spression()],
            },
            Decl::Fn {
                ident,
                type_def: _, // TODO: Should not ignore here
                args,
                body,
            } => {
                let children = vec![
                    Spression {
                        node: "Args".to_string(),
                        span: None,
                        data: Vec::new(),
                        children: args.into_iter().map(|a| a.to_spression()).collect(),
                    },
                    Spression {
                        node: "Body".to_string(),
                        span: None,
                        data: Vec::new(),
                        children: vec![body.to_spression()],
                    },
                ];

                Spression {
                    node: "Fn".to_string(),
                    span: None,
                    data: vec![format!("\"{}\"", ident.to_string())],
                    children,
                }
            }
        }
    }
}
