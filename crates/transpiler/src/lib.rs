use japp_util::Spanned;
use parser::{Decl, Expr, FnRow, Lit, Program};

struct Idents {}

impl Idents {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&mut self, s: &str) -> String {
        // TODO: Does javascript support utf-8 symbols?
        s.chars()
            .map(|c| if c.is_ascii() { c } else { c })
            .collect()
    }
}

pub fn transpile(mut program: Program) -> String {
    let mut out = String::new();

    let mut idents = Idents::new();

    for (_, decl) in program.declarations.drain() {
        match decl {
            Decl::Const { ident, rhs } => {
                let ident = idents.get(ident.outer());
                out.push_str(
                    format!("const {ident} = {};\n", transpile_expr(&mut idents, rhs)).as_str(),
                );
            }
            Decl::Fn {
                ident,
                mut rows,
                type_def: _,
            } => {
                let ident = idents.get(ident.outer());
                if rows.is_empty() {
                    return out;
                }

                if rows[0].args.len() == 0 {
                    out.push_str(
                        format!(
                            "const {ident} = () => {{ 
                                {}
                            }}; \n",
                            transpile_expr(&mut idents, rows.pop().unwrap().body),
                        )
                        .as_str(),
                    );
                } else {
                    // println!("{} {}", rows.len(), rows[0].args.len());
                    // TODO: This is stupid! Only works for |args| = 1
                    let args = (0..)
                        .take(rows[0].args.len())
                        .map(|n| format!("a{n}"))
                        .collect::<Vec<_>>()
                        .join(", ");

                    out.push_str(&format!(
                        "\nconst {ident} = ({args}) => {{ 
                            switch ({args}) {{
                                {}
                            }}
                        }}; \n",
                        rows.into_iter()
                            .map(|row| transpile_fn_row(&mut idents, row))
                            .collect::<String>()
                    ));
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
                idents.get(op.inner()),
                transpile_expr(idents, *rhs)
            )
        }
        Expr::FCall { ident, args } => {
            let mut ident = idents.get(ident.outer());
            if ident == "println" {
                ident = "console.log".to_string();
            }

            format!(
                "( {ident}({}) )",
                args.into_iter()
                    .map(|e| transpile_expr(idents, e))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        Expr::Assign { ident, expr } => {
            format!("{} = {expr} ;", idents.get(ident.outer()))
        }
        Expr::Lit(lit) => transpile_lit(idents, lit),
        Expr::Prefix { op, rhs } => format!(
            "( {} {} )",
            idents.get(op.inner()),
            transpile_expr(idents, *rhs)
        ),
        Expr::Block { exprs, last } => transpile_block(idents, exprs, last),
    }
}

fn transpile_lit(idents: &mut Idents, lit: Spanned<Lit>) -> String {
    match lit.inner {
        Lit::Null => "null".to_string(),
        Lit::Bool(b) => b.to_string(),
        Lit::Int(n) => n.to_string(),
        Lit::Ident(i) => idents.get(i.outer()),
    }
}

fn transpile_fn_row(idents: &mut Idents, body: FnRow) -> String {
    let ident = match body.args[0].inner() {
        Lit::Null => todo!(),
        Lit::Bool(i) => format!("case {i}:"),
        Lit::Int(i) => format!("case {i}: "),
        Lit::Ident(i) => format!("default:\n\tlet {} = a0;", idents.get(i.outer())),
    };

    format!("{ident} return {}; ", transpile_expr(idents, body.body))
}

fn transpile_block(idents: &mut Idents, exprs: Vec<Expr>, last: Option<Box<Expr>>) -> String {
    let mut s = String::new();

    for e in exprs {
        s += &transpile_expr(idents, e);
        s += ";\n";
    }

    if let Some(last) = last {
        s += &format!("return {} ;", transpile_expr(idents, *last));
    }

    format!("{{\n{s}}}\n")
}
