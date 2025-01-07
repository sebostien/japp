use crate::expr::Expr;

#[derive(Debug, PartialEq)]
pub enum Decl {
    Expr(Expr),
    Let {
        ident: String,
        rhs: Expr,
    },
    Fn {
        ident: String,
        args: Vec<String>,
        body: Expr,
    },
}

impl std::fmt::Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expr(e) => write!(f, "{e};"),
            Self::Let { ident, rhs } => {
                write!(f, "let {ident} = {rhs};")
            }
            Self::Fn { ident, args, body } => {
                let args = args.join(" ");
                write!(f, "fn {ident} {args} = {body};")
            }
        }
    }
}
