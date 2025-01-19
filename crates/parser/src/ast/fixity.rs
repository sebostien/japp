#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Fixity {
    pub prec: usize,
    pub assoc: Assoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Assoc {
    /// Left associativity
    ///
    /// `1 / 2 / 3 = (1 / 2) / 3`
    Left,
    /// Right associativity
    ///
    /// `1^2^3 = 1^(2^3)`
    Right,
    /// No associativity
    ///
    /// Ok : "1 == (2 == 3)"
    /// Err: "1 == 2 == 3"
    None,
}
