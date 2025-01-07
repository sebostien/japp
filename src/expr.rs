use std::str::FromStr;

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
    Let {
        name: String,
        rhs: Box<Expr>,
        then: Box<Expr>,
    },
    Fn {
        name: String,
        args: Vec<String>,
        body: Box<Expr>,
        then: Box<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Ident(ident) => write!(f, "{ident}"),
            Expr::Neg(expr) => write!(f, "-{expr}"),
            Expr::BinOp { lhs, op, rhs } => write!(f, "({lhs} {op} {rhs})"),
            Expr::Call(name, ops) => {
                let ops = ops
                    .into_iter()
                    .map(Expr::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                writeln!(f, "({name})({ops})")
            }
            Expr::Let { name, rhs, then } => {
                writeln!(f, "let {name} = {rhs};")?;
                write!(f, "{then}")
            }
            Expr::Fn {
                name,
                args,
                body,
                then,
            } => {
                let args = args.join(" ");

                writeln!(f, "fn {name} {args} = {body};")?;
                write!(f, "{then}")
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
