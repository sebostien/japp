#[derive(Debug, PartialEq)]
pub enum Expr {
    Ident(String),
    Neg(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    Call(Box<Expr>, Vec<Expr>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
            Self::Neg(expr) => write!(f, "-{expr}"),
            Self::BinOp { lhs, op, rhs } => write!(f, "({lhs} {op} {rhs})"),
            Self::Call(name, ops) => {
                let ops = ops
                    .into_iter()
                    .map(Expr::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "({name})({ops})")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => "+".fmt(f),
            BinOp::Sub => "-".fmt(f),
            BinOp::Mul => "*".fmt(f),
            BinOp::Div => "/".fmt(f),
        }
    }
}

impl TryFrom<char> for BinOp {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Self::Add),
            '-' => Ok(Self::Sub),
            '*' => Ok(Self::Mul),
            '/' => Ok(Self::Div),
            _ => Err("Invalid binary operand"),
        }
    }
}
