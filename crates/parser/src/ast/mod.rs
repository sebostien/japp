mod decl;
mod expr;
mod fixity;
mod lit;

pub use decl::{Decl, UnparsedDecl};
pub use expr::Expr;
pub use fixity::{Assoc, Fixity};
pub use lit::Lit;

#[derive(Debug)]
pub struct Program<'a> {
    pub declarations: Vec<Decl<'a>>,
}

impl std::fmt::Display for Program<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.declarations
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .fmt(f)
    }
}

#[derive(Debug, PartialEq)]
pub struct UnparsedProgram<'a> {
    pub decls: Vec<UnparsedDecl<'a>>,
}

impl std::fmt::Display for UnparsedProgram<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.decls
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .fmt(f)
    }
}
