mod decl;
mod expr;
mod fixity;
mod lit;
mod spanned;

pub use decl::{Decl, UnparsedDecl};
pub use expr::Expr;
pub use fixity::{Assoc, Fixity};
pub use lit::Lit;
pub use spanned::Spanned;

#[derive(Debug)]
pub struct Program {
    pub declarations: Vec<Decl>,
}

#[derive(Debug, PartialEq)]
pub struct UnparsedProgram {
    pub decls: Vec<UnparsedDecl>,
}

impl std::fmt::Display for UnparsedProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.decls
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .fmt(f)
    }
}
