#[derive(Debug, PartialEq)]
pub struct Program {
    pub exprs: Vec<Expr>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.exprs.iter().map(|e| writeln!(f, "{e}")).collect()
    }
}

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
    },
    Fn {
        name: String,
        args: Vec<String>,
        body: Box<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Ident(ident) => ident.fmt(f),
            Expr::Neg(expr) => write!(f, "-{expr}"),
            Expr::BinOp { lhs, op, rhs } => write!(f, "({lhs} {op} {rhs})"),
            Expr::Call(name, ops) => {
                let ops = ops
                    .into_iter()
                    .map(Expr::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "({name})({ops})")
            }
            Expr::Let { name, rhs } => {
                write!(f, "let {name} = {rhs};")
            }
            Expr::Fn { name, args, body } => {
                let args = args.join(" ");
                write!(f, "fn {name} {args} = {body};")
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
