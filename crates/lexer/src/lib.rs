mod interleave;
mod nfa;
mod state;
mod token;

use japp_util::Spanned;
use std::str::Chars;

use interleave::Interleave;
use nfa::Nfa;
use token::Token;

#[derive(Debug)]
pub struct Lexer {
    nfa: Nfa,
}

impl Lexer {
    pub fn compile<I: IntoIterator<Item = S>, S: AsRef<str>>(symbols: I) -> Self {
        let symbols = symbols
            .into_iter()
            .map(|symbol| {
                println!("{}", symbol.as_ref());
                symbol
                    .as_ref()
                    .chars()
                    .map(Token::Char)
                    .interleave((0..).map(|_| Token::Concat))
                    .collect()
            })
            .interleave((0..).map(|_| vec![Token::Union]))
            .flatten();

        Self {
            nfa: Nfa::compile(symbols).unwrap(),
        }
    }

    #[must_use]
    pub fn find<I: Iterator<Item = char>>(&self, input: I) -> Option<usize> {
        self.nfa.find(input)
    }

    #[must_use]
    pub fn find_match<'a>(&self, input: &'a str) -> Option<&'a str> {
        Some(&input[0..self.nfa.find(input.chars())?])
    }

    #[must_use]
    pub fn scan<'source>(&self, offset: usize, input: &'source str) -> Scanner<'_, 'source> {
        Scanner {
            nfa: &self.nfa,
            source: input,
            chars: input.chars(),
            next: None,
            offset,
        }
    }
}

pub struct Scanner<'l, 'source> {
    nfa: &'l Nfa,
    source: &'source str,
    chars: Chars<'source>,
    next: Option<Spanned<&'source str>>,
    offset: usize,
}

impl<'source> Iterator for Scanner<'_, 'source> {
    type Item = Spanned<&'source str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_some() {
            return self.next.take();
        }

        let mut no_match_end = 0;

        while let Some(c) = self.chars.next() {
            let start = self.offset;
            let first = &self.source[0..no_match_end];

            if c.is_whitespace() {
                if no_match_end > 0 {
                    let first = &self.source[0..no_match_end];

                    self.source = &self.source[no_match_end + c.len_utf8()..];
                    self.offset += no_match_end + c.len_utf8();

                    return Some(Spanned {
                        span: start..start + no_match_end,
                        inner: first,
                    });
                }

                // Still at start, just skip the first whitespace
                self.source = &self.source[c.len_utf8()..];
                self.offset += c.len_utf8();
            } else if let Some(m) = self.nfa.find(&mut self.source[no_match_end..].chars()) {
                let middle = self.offset + no_match_end;
                let second = &self.source[no_match_end..no_match_end + m];
                let source = self.source;

                self.source = &self.source[no_match_end + m..];
                self.offset += no_match_end + m;

                // Move the char iter to end of match.
                for _ in c.len_utf8()..m {
                    let _ = self.chars.next();
                }

                // If we have chars before the start of the match we
                // need to return them first.
                if !first.is_empty() {
                    self.next = Some(Spanned {
                        span: middle..self.offset,
                        inner: second,
                    });

                    return Some(Spanned {
                        span: start..middle,
                        inner: first,
                    });
                } else {
                    return Some(Spanned {
                        span: start..self.offset,
                        inner: &source[0..m],
                    });
                }
            } else {
                no_match_end += c.len_utf8();
            }
        }

        if no_match_end > 0 {
            let start = self.offset;
            self.offset += no_match_end;

            return Some(Spanned {
                span: start..self.offset,
                inner: &self.source[0..no_match_end],
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::Lexer;

    impl std::fmt::Display for Lexer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.nfa.fmt(f)
        }
    }
}
