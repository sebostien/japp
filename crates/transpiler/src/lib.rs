use japp_util::Spanned;
use parser::{Decl, Expr, Ident, Lit, Program};

struct Idents {}

impl Idents {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&mut self, s: &str) -> String {
        // TODO: Actually make functions work
        s.to_string()
    }
}

trait Transpile {
    type Out;

    fn transpile(self, idents: &mut Idents, indent: usize) -> Self::Out;
}

pub fn transpile(mut program: Program) -> String {
    let mut out = String::new();
    let mut idents = Idents::new();

    for (_, decl) in program.declarations.drain() {
        if let Some(decl) = decl.transpile(&mut idents, 0) {
            out += &decl;
            out += "\n";
        }
    }

    out.push_str("main();\n");

    out
}

impl Transpile for Decl<'_> {
    type Out = Option<String>;

    fn transpile(self, idents: &mut Idents, indent: usize) -> Self::Out {
        match self {
            Decl::Const { ident, rhs } => {
                let ident = idents.get(ident.outer());
                Some(format!(
                    "const {ident} = {};\n",
                    rhs.transpile(idents, indent)
                ))
            }
            Decl::Fn {
                ident,
                type_def: _,
                args,
                body,
            } => {
                let ident = idents.get(ident.outer());

                if args.is_empty() {
                    Some(format!(
                        "const {ident} = () => {};\n",
                        body.transpile(idents, indent)
                    ))
                } else {
                    let args = args.iter().map(Ident::inner).collect::<Vec<_>>().join(", ");

                    let mut out = String::new();

                    out += &format!("const {ident} = ({args}) => {{\n");
                    out += &body.transpile(idents, indent + 4);
                    out += "};\n";

                    Some(out)
                }
            }
        }
    }
}

impl Transpile for Expr<'_> {
    type Out = String;

    fn transpile(self, idents: &mut Idents, indent: usize) -> Self::Out {
        match self {
            Expr::Binary { lhs, op, rhs } => {
                format!(
                    "({} {} {})",
                    lhs.transpile(idents, indent),
                    idents.get(op.inner()),
                    rhs.transpile(idents, indent)
                )
            }
            Expr::Match { var, body } => {
                let mut out = String::new();
                out += &format!("{}switch ({var}) {{\n", " ".repeat(indent));

                let spaces = " ".repeat(indent + 4);

                for (pat, expr) in &body.cases {
                    match pat {
                        parser::Pattern::Lit(Lit::Null) => {
                            out += &spaces;
                            out += "case null:\n";
                        }
                        parser::Pattern::Lit(Lit::Bool(i)) => {
                            out += &spaces;
                            out += if *i { "case true:\n" } else { "case false:\n" };
                        }
                        parser::Pattern::Lit(Lit::Int(i)) => {
                            out += &spaces;
                            out += &format!("case {i}:\n");
                        }
                        parser::Pattern::Lit(Lit::Ident(i)) => {
                            out += &spaces;
                            out += "default:\n";
                        }
                    }

                    out += &format!("{}return {expr};\n", " ".repeat(indent + 8));
                }

                out += &" ".repeat(indent);
                out += "};\n";

                out
            }
            Expr::FCall { ident, args } => {
                let mut ident = idents.get(ident.outer());
                if ident == "println" {
                    ident = "console.log".to_string();
                }

                format!(
                    "{ident}({})",
                    args.into_iter()
                        .map(|e| e.transpile(idents, indent))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Lit(lit) => lit.transpile(idents, indent),
            Expr::Prefix { op, rhs } => {
                format!(
                    "({}{})",
                    idents.get(op.inner()),
                    rhs.transpile(idents, indent)
                )
            }
            Expr::Block { exprs, last } => transpile_block(idents, exprs, last, indent),
        }
    }
}

impl<T: Transpile> Transpile for Spanned<T> {
    type Out = T::Out;

    fn transpile(self, idents: &mut Idents, indent: usize) -> Self::Out {
        self.inner.transpile(idents, indent)
    }
}

impl Transpile for Lit<'_> {
    type Out = String;

    fn transpile(self, idents: &mut Idents, _indent: usize) -> Self::Out {
        match self {
            Lit::Null => "null".to_string(),
            Lit::Bool(b) => b.to_string(),
            Lit::Int(n) => n.to_string(),
            Lit::Ident(i) => idents.get(i.outer()),
        }
    }
}

fn transpile_block(
    idents: &mut Idents,
    exprs: Vec<Expr>,
    last: Option<Box<Expr>>,
    indent: usize,
) -> String {
    let indent = indent + 4;
    let mut s = String::new();

    s += "{\n";

    for e in exprs {
        s += &" ".repeat(indent);
        s += &e.transpile(idents, indent + 4);
        s += ";\n";
    }

    if let Some(last) = last {
        s += &" ".repeat(indent);
        s += "return ";
        s += &last.transpile(idents, indent + 4);
        s += ";\n";
    }

    s += "}";
    s
}
