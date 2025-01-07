use crate::{decl::Decl, expr::Expr};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub exprs: Vec<Decl>,
    pub last: Option<Expr>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut exprs = self.exprs
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>();


        if let Some(last) = &self.last {
            exprs.push(last.to_string());
        }

        write!(f, "{}", exprs.join("\n"))
    }
}
