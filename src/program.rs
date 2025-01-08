use crate::decl::Decl;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub exprs: Vec<Decl>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.exprs
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .fmt(f)
    }
}
