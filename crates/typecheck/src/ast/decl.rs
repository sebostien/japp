use crate::{ast::Typed, Type};

use super::{Expr};
use parser::Ident;

#[derive(Debug)]
pub enum Decl<'a> {
    Const {
        ident: Ident<'a>,
        rhs: Typed<Expr<'a>>,
    },
    Fn {
        ident: Ident<'a>,
        args: Vec<Type>,
        return_ty: Type,
        body: Typed<Expr<'a>>,
    },
}
