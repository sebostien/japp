use crate::ast::Typed;

use super::Lit;
use japp_util::Spanned;
use parser::Ident;

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    /// `x + y`
    /// `e = e`
    Binary {
        lhs: Box<Self>,
        op: Ident<'a>,
        rhs: Box<Self>,
    },
    /// `!x`
    Prefix { op: Ident<'a>, rhs: Box<Self> },
    /// `f(e1, e2)`
    FCall { ident: Ident<'a>, args: Vec<Typed<Self>> },
    /// `{ e1 ; e2 ; }`
    /// `{ e1 ; e2 ; last }`
    Block {
        exprs: Vec<Typed<Self>>,
        last: Option<Typed<Box<Self>>>,
    },
    /// `2`
    /// `true`
    Lit(Spanned<Lit<'a>>),
}
