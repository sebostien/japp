use super::Lit;

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
    Lit(Lit),
}
impl Expr {
    pub fn eval(self) -> Lit {
        match self {
            Expr::Binary { lhs, op, rhs } => match op.as_str() {
                "+" => Lit::Num(
                    isize::try_from(lhs.eval()).unwrap() + isize::try_from(rhs.eval()).unwrap(),
                ),
                "-" => Lit::Num(
                    isize::try_from(lhs.eval()).unwrap() - isize::try_from(rhs.eval()).unwrap(),
                ),
                "*" => Lit::Num(
                    isize::try_from(lhs.eval()).unwrap() * isize::try_from(rhs.eval()).unwrap(),
                ),
                "/" => Lit::Num(
                    isize::try_from(lhs.eval()).unwrap() / isize::try_from(rhs.eval()).unwrap(),
                ),
                "^" => Lit::Num(
                    isize::try_from(lhs.eval())
                        .unwrap()
                        .pow(isize::try_from(rhs.eval()).unwrap() as u32),
                ),
                "==" => Lit::Bool(
                    isize::try_from(lhs.eval()).unwrap() == rhs.eval().try_into().unwrap(),
                ),
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
                    let a = isize::try_from(args.pop().unwrap().eval()).unwrap();
                    let b = isize::try_from(args.pop().unwrap().eval()).unwrap();
                    Lit::Num(a + b)
                }
                _ => panic!("Unknown function {ident}"),
            },
            Expr::Lit(l) => l,
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary { lhs, op, rhs } => write!(f, "( {lhs} {op} {rhs} )"),
            Expr::Lit(lit) => lit.fmt(f),
            Expr::FCall { ident, args } => {
                write!(f, "{ident} ( {} )", args[0])
            }
        }
    }
}
