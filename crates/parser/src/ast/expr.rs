use core::fmt;
use japp_util::Spanned;
use spressions::{Spression, ToSpression};

use super::{Ident, Lit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'a> {
    /// `x + y`
    /// `e = e`
    Binary {
        lhs: Box<Self>,
        op: Ident<'a>,
        rhs: Box<Self>,
    },
    Match {
        var: Box<Self>,
        body: MatchBody<'a>,
    },
    /// `!x`
    Prefix {
        op: Ident<'a>,
        rhs: Box<Self>,
    },
    /// `f(e1, e2)`
    FCall {
        ident: Ident<'a>,
        args: Vec<Self>,
    },
    /// `{ e1 ; e2 ; }`
    /// `{ e1 ; e2 ; last }`
    Block {
        exprs: Vec<Self>,
        last: Option<Box<Self>>,
    },
    /// `2`
    /// `true`
    Lit(Spanned<Lit<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchBody<'a> {
    pub cases: Vec<(Pattern<'a>, Expr<'a>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<'a> {
    Lit(Lit<'a>),
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
            Expr::Block { exprs, last } => {
                for e in exprs {
                    e.eval()?;
                }

                if let Some(last) = last {
                    last.eval()
                } else {
                    Ok(Lit::Null)
                }
            }
            Expr::Lit(l) => Ok(l.inner),
            Expr::Match { .. } => todo!(),
        }
    }
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary { lhs, op, rhs } => write!(f, "( {lhs} {} {rhs} )", op.inner()),
            Self::Match { var, body } => write!(f, "match {var} {{ {body}}}"),
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

impl fmt::Display for MatchBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (pat, body) in &self.cases {
            write!(f, "{pat} -> {body} ; ")?;
        }

        Ok(())
    }
}

impl fmt::Display for Pattern<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Lit(lit) => lit.fmt(f),
        }
    }
}

impl<'a> ToSpression for Expr<'a> {
    fn to_spression(self) -> Spression {
        match self {
            Expr::Binary { lhs, op, rhs } => Spression {
                node: op.outer.to_string(),
                span: None,
                data: Vec::new(),
                children: vec![lhs.to_spression(), rhs.to_spression()],
            },
            Expr::Match { var, body } => todo!(),
            Expr::Prefix { op, rhs } => todo!(),
            Expr::FCall { ident, args } => {
                let mut children = vec![ident.to_spression()];
                children.append(&mut args.into_iter().map(|a| a.to_spression()).collect());
                Spression {
                    node: "FCall".to_string(),
                    span: None,
                    data: Vec::new(),
                    children,
                }
            }
            Expr::Block { exprs, last } => {
                let mut children = exprs
                    .into_iter()
                    .map(Expr::to_spression)
                    .collect::<Vec<_>>();

                if let Some(last) = last {
                    children.push(Spression {
                        node: "Last".to_string(),
                        span: None,
                        data: Vec::new(),
                        children: vec![last.to_spression()],
                    });
                }

                Spression {
                    node: "Block".to_string(),
                    span: None,
                    data: Vec::new(),
                    children,
                }
            }
            Expr::Lit(Spanned { span, inner }) => Spression {
                node: "Lit".to_string(),
                span: Some(span),
                data: Vec::new(),
                children: vec![inner.to_spression()],
            },
        }
    }
}
