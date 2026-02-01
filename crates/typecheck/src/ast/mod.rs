mod decl;
mod expr;
mod fixity;
mod lit;

pub use decl::Decl;
pub use expr::Expr;
pub use fixity::{Associativity, Fixity};
pub use lit::Lit;

use super::Type;

#[derive(Debug)]
pub struct TypedProgram<'a> {
    pub declarations: Vec<Decl<'a>>,
}

#[derive(Debug, Clone)]
pub struct Typed<T> {
    pub ty: Type,
    pub inner: T,
}

impl<T> Typed<T> {
    pub fn new(ty: Type, inner: T) -> Self {
        Self { ty, inner }
    }
}
