#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Fixity {
    pub prec: usize,
    pub assoc: Associativity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Associativity {
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
