use super::{Ident, Lit};
use japp_util::Spanned;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'a> {
    Binary {
        lhs: Box<Self>,
        op: Ident<'a>,
        rhs: Box<Self>,
    },
    Prefix {
        op: Ident<'a>,
        rhs: Box<Self>,
    },
    FCall {
        ident: Ident<'a>,
        args: Vec<Self>,
    },
    Block {
        exprs: Vec<Self>,
        last: Option<Box<Self>>,
    },
    Lit(Spanned<Lit<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EvalError<'a> {
    ExpectedInt(Lit<'a>),
    ExpectedBool(Lit<'a>),
}

impl<'a> Expr<'a> {
    pub fn eval(self) -> Result<Lit<'a>, EvalError<'a>> {
        match self {
            Expr::Binary { lhs, op, rhs } => match op.inner {
                "+" => Ok(Lit::Int(
                    isize::try_from(lhs.eval()?)? + isize::try_from(rhs.eval()?)?,
                )),
                "-" => Ok(Lit::Int(
                    isize::try_from(lhs.eval()?)? - isize::try_from(rhs.eval()?)?,
                )),
                "*" => Ok(Lit::Int(
                    isize::try_from(lhs.eval()?)? * isize::try_from(rhs.eval()?)?,
                )),
                "/" => Ok(Lit::Int(
                    isize::try_from(lhs.eval()?)? / isize::try_from(rhs.eval()?)?,
                )),
                "^" => Ok(Lit::Int(
                    isize::try_from(lhs.eval()?)?.pow(isize::try_from(rhs.eval()?)? as u32),
                )),
                "==" => Ok(Lit::Bool(lhs.eval()? == rhs.eval()?)),
                _ => unreachable!(),
            },
            Expr::Prefix { op, rhs } => match op.inner {
                "!" => Ok(Lit::Bool(!bool::try_from(rhs.eval()?)?)),
                _ => unreachable!(),
            },
            Expr::FCall { ident, mut args } => match ident.inner {
                "identity" => {
                    assert_eq!(
                        args.len(),
                        1,
                        "The identity function takes exactly one argument"
                    );
                    args.pop().unwrap().eval()
                }
                "add" => {
                    assert_eq!(
                        args.len(),
                        2,
                        "The add function takes exactly two arguments"
                    );
                    let a = isize::try_from(args.pop().unwrap().eval()?)?;
                    let b = isize::try_from(args.pop().unwrap().eval()?)?;
                    Ok(Lit::Int(a + b))
                }
                _ => panic!("Unknown function {ident:?}"),
            },
            Expr::Block { .. } => {
                todo!()
            }
            Expr::Lit(l) => Ok(l.inner),
        }
    }
}

impl std::fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary { lhs, op, rhs } => write!(f, "( {lhs} {} {rhs} )", op.inner()),
            Self::Prefix { op, rhs } => write!(f, "( {} {rhs} )", op.inner()),
            Self::Lit(lit) => lit.fmt(f),
            Self::FCall { ident, args } => {
                write!(f, "{} (", ident.outer())?;

                if !args.is_empty() {
                    let last = args.len() - 1;
                    for (i, arg) in args.iter().enumerate() {
                        if i == last {
                            write!(f, " {arg}")?;
                        } else {
                            write!(f, " {arg} ,")?;
                        }
                    }
                }

                write!(f, " )")
            }
            Self::Block { exprs, last } => {
                write!(f, "{{")?;

                for e in exprs {
                    write!(f, " {e} ;")?;
                }

                if let Some(e) = last {
                    write!(f, " {e} ")?;
                }

                write!(f, "}}")
            }
        }
    }
}
