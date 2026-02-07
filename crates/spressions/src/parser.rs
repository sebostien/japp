use std::ops::Range;

use crate::Spression;

pub struct Parser<'s> {
    input: &'s str,
    pos: usize,
}

type Result<T> = std::result::Result<T, String>;

impl<'s> Parser<'s> {
    pub fn new(input: &'s str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// Consume next token if it matches the expected
    fn expect(&mut self, expected: char) -> Result<()> {
        let found = self.peek().ok_or("EOF".to_string())?;

        if found != expected {
            return Err(format!(
                "Unexpected token '{found}', expected: '{expected}' at {}",
                self.pos
            ));
        }

        self.consume();
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    /// Parse a single spression
    pub fn expr(&mut self) -> Result<Option<Spression>> {
        // Start
        self.skip_whitespace();
        if self.expect('(').is_err() {
            return Ok(None);
        }

        // Parse inside expression
        let node = self.ident()?.to_string();

        let mut data = Vec::new();

        while let Ok(d) = self.ident() {
            data.push(d.to_string());
        }

        let mut children = Vec::new();

        while let Some(e) = self.expr()? {
            children.push(e);
        }

        println!("After children in node {node:#?}");

        // Done
        self.skip_whitespace();
        self.expect(')')?;

        println!("After ')' in {node:#?}");

        let span = self.span()?;

        Ok(Some(Spression {
            node,
            span,
            data,
            children,
        }))
    }

    fn ident(&mut self) -> Result<&'s str> {
        self.skip_whitespace();
        let start = self.pos;

        while let Some(c) = self.peek() {
            if c.is_whitespace() || matches!(c, '(' | ')' | ':') {
                break;
            }

            self.consume();
        }

        if start == self.pos {
            Err("EOF".to_string())
        } else {
            Ok(&self.input[start..self.pos])
        }
    }

    fn span(&mut self) -> Result<Option<Range<usize>>> {
        self.skip_whitespace();
        if self.expect(':').is_err() {
            return Ok(None);
        }

        println!("In range, after :");

        let start = self.int()?;

        self.skip_whitespace();
        self.expect('.')?;
        self.expect('.')?;

        let end = self.int()?;

        Ok(Some(start..end))
    }

    fn int(&mut self) -> Result<usize> {
        self.skip_whitespace();
        let start = self.pos;

        while let Some(i) = self.peek() {
            if i.is_whitespace() || matches!(i, '(' | ')' | '.') {
                break;
            }
            self.consume();
        }

        if start == self.pos {
            Err("Expected number".to_string())
        } else {
            self.input[start..self.pos]
                .parse::<usize>()
                .map_err(|e| e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn range() {
        let s = ":10..80";
        assert_eq!(Parser::new(s).span(), Ok(Some(10..80)));

        let s = " : 10 .. 80";
        assert_eq!(Parser::new(s).span(), Ok(Some(10..80)));

        let s = ":0..0";
        assert_eq!(Parser::new(s).span(), Ok(Some(0..0)));

        let s = "10..80";
        assert_eq!(Parser::new(s).span(), Ok(None));

        let s = ":10.80";
        assert!(Parser::new(s).span().is_err());

        let s = ":10..";
        assert!(Parser::new(s).span().is_err());

        let s = ":..10";
        assert!(Parser::new(s).span().is_err());
    }

    #[test]
    fn int() {
        let s = "  1234 ";
        assert_eq!(Parser::new(s).int(), Ok(1234));

        let s = "1234   ";
        assert_eq!(Parser::new(s).int(), Ok(1234));

        let s = "   1234";
        assert_eq!(Parser::new(s).int(), Ok(1234));

        let s = " 12 ab";
        assert_eq!(Parser::new(s).int(), Ok(12));

        let s = "abc";
        assert!(Parser::new(s).int().is_err());

        let s = " 1234abc";
        assert!(Parser::new(s).int().is_err());
    }
}
