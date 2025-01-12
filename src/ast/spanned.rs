use std::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Spanned<T> {
    pub span: Range<usize>,
    pub inner: T,
}

impl<T: std::fmt::Display> std::fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
