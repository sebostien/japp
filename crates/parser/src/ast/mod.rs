mod decl;
mod expr;
mod fixity;
mod lit;

use std::collections::HashMap;

pub use decl::{Decl, FnRow, UnparsedDecl};
pub use expr::Expr;
pub use fixity::{Assoc, Fixity};
pub use lit::Lit;

#[derive(Debug)]
pub struct Program<'a> {
    pub declarations: HashMap<&'a str, Decl<'a>>,
}

impl std::fmt::Display for Program<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut decls = self.declarations.iter().collect::<Vec<_>>();
        decls.sort_by_key(|(k, _)| *k);

        decls
            .into_iter()
            .map(|(_, v)| v)
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
