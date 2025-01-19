use std::collections::HashMap;

use japp_util::Spanned;
use parser::{
    ast::{Expr, Lit},
    Decl, FnRow, Program,
};

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

pub fn transpile(mut program: Program) -> String {
    let mut out = String::new();

    let mut idents = Idents::new();

    for (_, decl) in program.declarations.drain() {
        match decl {
            Decl::Let { ident, rhs } => {
                let ident = idents.get(ident);
                out.push_str(
                    format!("let {ident} = {};\n", transpile_expr(&mut idents, rhs)).as_str(),
                );
            }
            Decl::Fn { ident, mut rows } => {
                let ident = idents.get(ident);

                if rows[0].args.len() == 0 {
                    out.push_str(
                        format!(
                            "let {ident} = () => {{ 
                                {}
                            }}; \n",
                            transpile_expr(&mut idents, rows.pop().unwrap().body),
                        )
                        .as_str(),
                    );
                } else {
                    println!("{} {}", rows.len(), rows[0].args.len());
                    // TODO: This is stupid! Only works for |args| = 1
                    let args = (0..)
                        .take(rows[0].args.len())
                        .map(|n| format!("a{n}"))
                        .collect::<Vec<_>>()
                        .join(", ");

                    out.push_str(
                        format!(
                            "let {ident} = ({args}) => {{ 
                            switch ({args}) {{
                                {}
                            }}
                        }}; \n",
                            rows.into_iter()
                                .map(|row| transpile_fn_row(&mut idents, row))
                                .collect::<String>()
                        )
                        .as_str(),
                    );
                }
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
        Expr::Lit(lit) => transpile_lit(idents, lit),
    }
}

fn transpile_lit(idents: &mut Idents, lit: Spanned<Lit>) -> String {
    match lit.inner {
        Lit::Bool(b) => b.to_string(),
        Lit::Num(n) => n.to_string(),
        Lit::Ident(i) => idents.get(i),
    }
}

fn transpile_fn_row(idents: &mut Idents, body: FnRow) -> String {
    let ident = match body.args[0].inner {
        Lit::Bool(i) => format!("case {i}:"),
        Lit::Num(i) => format!("case {i}: "),
        Lit::Ident(i) => format!("default:\n\tlet {} = a0;", idents.get(i)),
    };

    format!("{ident} return {}; ", transpile_expr(idents, body.body))
}
