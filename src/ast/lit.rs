#[derive(Debug, PartialEq, Eq)]
pub enum Lit {
    Bool(bool),
    Num(isize),
    Ident(String),
}

impl std::fmt::Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => b.fmt(f),
            Self::Num(i) => i.fmt(f),
            Self::Ident(i) => i.fmt(f),
        }
    }
}

impl<S: AsRef<str>> From<S> for Lit {
    fn from(s: S) -> Self {
        let s = s.as_ref();
        if s == "true" {
            Lit::Bool(true)
        } else if s == "false" {
            Lit::Bool(false)
        } else if let Ok(num) = s.parse() {
            Lit::Num(num)
        } else {
            Lit::Ident(s.to_string())
        }
    }
}

impl TryFrom<Lit> for isize {
    type Error = ();

    fn try_from(value: Lit) -> Result<Self, Self::Error> {
        if let Lit::Num(n) = value {
            Ok(n)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Lit> for bool {
    type Error = ();

    fn try_from(value: Lit) -> Result<Self, Self::Error> {
        if let Lit::Bool(b) = value {
            Ok(b)
        } else {
            Err(())
        }
    }
}
