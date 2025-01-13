use super::{Lit, Spanned};

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Binary {
        lhs: Box<Expr>,
        op: String,
        rhs: Box<Expr>,
    },
    FCall {
        ident: String,
        args: Vec<Expr>,
    },
    Lit(Spanned<Lit>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EvalError {
    ExpectedInt(Lit),
}

impl Expr {
    pub fn eval(self) -> Result<Lit, EvalError> {
        match self {
            Expr::Binary { lhs, op, rhs } => match op.as_str() {
                "+" => Ok(Lit::Num(
                    isize::try_from(lhs.eval()?)? + isize::try_from(rhs.eval()?)?,
                )),
                "-" => Ok(Lit::Num(
                    isize::try_from(lhs.eval()?)? - isize::try_from(rhs.eval()?)?,
                )),
                "*" => Ok(Lit::Num(
                    isize::try_from(lhs.eval()?)? * isize::try_from(rhs.eval()?)?,
                )),
                "/" => Ok(Lit::Num(
                    isize::try_from(lhs.eval()?)? / isize::try_from(rhs.eval()?)?,
                )),
                "^" => Ok(Lit::Num(
                    isize::try_from(lhs.eval()?)?.pow(isize::try_from(rhs.eval()?)? as u32),
                )),
                "==" => Ok(Lit::Bool(lhs.eval()? == rhs.eval()?)),
                _ => unreachable!(),
            },
            Expr::FCall { ident, mut args } => match ident.as_str() {
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
                    Ok(Lit::Num(a + b))
                }
                _ => panic!("Unknown function {ident}"),
            },
            Expr::Lit(l) => Ok(l.inner),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary { lhs, op, rhs } => write!(f, "( {lhs} {op} {rhs} )"),
            Expr::Lit(lit) => lit.fmt(f),
            Expr::FCall { ident, args } => {
                write!(f, "{ident} (")?;

                let last = args.len().wrapping_sub(1);
                for (i, arg) in args.into_iter().enumerate() {
                    if i == last {
                        write!(f, " {arg}")?;
                    } else {
                        write!(f, " {arg} ,")?;
                    }
                }

                write!(f, " )")
            }
        }
    }
}
