use std::collections::HashMap;
use std::ops::{Deref, Range};

mod decl;
mod expr;
mod fixity;
mod lit;

pub use decl::{Decl, Type, UnparsedDecl};
pub use expr::{Expr, MatchBody, Pattern};
pub use fixity::{Associativity, Fixity};
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

#[derive(Debug, Clone, Eq)]
pub struct Ident<'a> {
    /// The whole ident as typed.
    /// E.g. `_+_`
    outer: &'a str,
    outer_span: Range<usize>,
    /// The inner part of the ident.
    /// E.g. `+` if `outer = _+_`
    inner: &'a str,
    inner_span: Range<usize>,
}

impl<'a> PartialEq for Ident<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<'a> Deref for Ident<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> Ident<'a> {
    pub fn new(span: Range<usize>, source: &'a str) -> Self {
        let start_trimmed = source.trim_start_matches('_');
        let end_trimmed = start_trimmed.trim_end_matches('_');

        let inner_start = span.start + (source.len() - start_trimmed.len());
        let inner_end = span.end - (start_trimmed.len() - end_trimmed.len());

        Self {
            outer: source,
            outer_span: span,
            inner: source.trim_matches('_'),
            inner_span: inner_start..inner_end,
        }
    }

    pub fn outer(&self) -> &'a str {
        self.outer
    }

    pub fn outer_span(&self) -> Range<usize> {
        self.outer_span.clone()
    }

    pub fn inner(&self) -> &'a str {
        self.inner
    }

    pub fn inner_span(&self) -> Range<usize> {
        self.inner_span.clone()
    }
}
