use std::collections::HashMap;

use japp_util::Spanned;
use parser::{ast::Expr, Decl, Program};

struct Idents {
    mapped: HashMap<String, String>,
    s: String,
}

impl Idents {
    pub fn new() -> Self {
        Self {
            mapped: HashMap::new(),
            s: String::new(),
        }
    }

    pub fn get(&mut self, s: &str) -> String {
        if is_builtin(s) {
            s.to_string()
        } else if let Some(s) = self.mapped.get(s) {
            s.clone()
        } else {
            self.s.push('a');
            self.mapped.insert(s.to_string(), self.s.clone());
            self.mapped.get(s).unwrap().clone()
        }
    }
}

fn is_builtin(s: &str) -> bool {
    matches!(s, "main" | "console.log" | "+" | "-" | "/" | "*" | "==")
}

pub fn transpile(program: Program) -> String {
    let mut out = String::new();

    let mut idents = Idents::new();

    for decl in program.declarations {
        match decl {
            Decl::Let { ident, rhs } => {
                let ident = idents.get(ident);
                out.push_str(
                    format!("let {ident} = {};\n", transpile_expr(&mut idents, rhs)).as_str(),
                );
            }
            Decl::Fn { ident, args, body } => {
                let ident = idents.get(ident);

                out.push_str(
                    format!(
                        "let {ident} = ({}) => {{ return {}; }}; \n",
                        transpile_args(&mut idents, args),
                        transpile_expr(&mut idents, body),
                    )
                    .as_str(),
                );
            }
        }
    }

    out.push_str("main();\n");

    out
}

fn transpile_expr(idents: &mut Idents, expr: Expr) -> String {
    match expr {
        Expr::Binary { lhs, op, rhs } => {
            format!(
                "({} {} {})",
                transpile_expr(idents, *lhs),
                idents.get(op.inner),
                transpile_expr(idents, *rhs)
            )
        }
        Expr::FCall { ident, args } => {
            let ident = idents.get(ident.inner);

            format!(
                "( {ident}({}) )",
                args.into_iter()
                    .map(|e| transpile_expr(idents, e))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        Expr::Lit(l) => match l.inner {
            parser::ast::Lit::Bool(b) => b.to_string(),
            parser::ast::Lit::Num(n) => n.to_string(),
            parser::ast::Lit::Ident(i) => idents.get(i),
        },
    }
}

fn transpile_args(idents: &mut Idents, args: Vec<Spanned<&str>>) -> String {
    args.into_iter()
        .map(|arg| idents.get(arg.inner))
        .collect::<Vec<_>>()
        .join(", ")
}
