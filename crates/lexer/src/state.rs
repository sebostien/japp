#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(pub usize);

impl PartialEq<usize> for State {
    fn eq(&self, other: &usize) -> bool {
        &self.0 == other
    }
}

impl PartialEq<State> for usize {
    fn eq(&self, other: &State) -> bool {
        self == &other.0
    }
}

impl From<usize> for State {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<State> for usize {
    fn from(value: State) -> Self {
        value.0
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
