use std::ops::Range;
use std::str::FromStr;

mod parser;

#[derive(Debug, Eq, Default)]
pub struct Spression {
    pub node: String,
    pub span: Option<Range<usize>>,
    pub data: Vec<String>,
    pub children: Vec<Spression>,
}

impl PartialEq for Spression {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
            && self.span == other.span
            && self.data.len() == other.data.len()
            && self.data.iter().all(|e| other.data.contains(e))
            && other.data.iter().all(|e| self.data.contains(e))
            && self.children.len() == other.children.len()
            && self.children.iter().all(|e| other.children.contains(e))
            && other.children.iter().all(|e| self.children.contains(e))
    }
}

pub trait ToSpression {
    fn to_spression(self) -> Spression;
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError(String);

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Spression {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parser::Parser::new(s)
            .expr()
            .map_err(ParseError)?
            .ok_or(ParseError("Expected expr".to_string()))
    }
}
