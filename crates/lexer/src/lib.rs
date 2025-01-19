mod interleave;
mod nfa;
mod state;
mod token;

use japp_util::Spanned;
use std::str::Chars;

use interleave::Interleave;
use nfa::Nfa;
use token::Token;

pub struct Lexer {
    nfa: Nfa,
}

impl Lexer {
    pub fn compile<I: IntoIterator<Item = S>, S: AsRef<str>>(symbols: I) -> Self {
        let symbols = symbols
            .into_iter()
            .map(|symbol| {
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

    pub fn scan<'source>(&self, offset: usize, input: &'source str) -> Scanner<'_, 'source> {
        Scanner {
            lexer: self,
            source: input,
            chars: input.chars(),
            next: None,
            offset,
        }
    }
}

pub struct Scanner<'l, 'source> {
    lexer: &'l Lexer,
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
                        span: start..self.offset,
                        inner: first,
                    });
                }

                // Still at start, just skip the first whitespace
                self.source = &self.source[c.len_utf8()..];
                self.offset += c.len_utf8();
            } else if let Some(m) = self.lexer.find(&mut self.source[no_match_end..].chars()) {
                let middle = self.offset + no_match_end;
                let second = &self.source[no_match_end..no_match_end + m];
                let inner = &self.source[0..m];

                self.source = &self.source[no_match_end + m..];
                self.offset += no_match_end + m;

                // Move our char iter to end of match.
                for _ in 1..m {
                    let _ = self.chars.next();
                }

                // If we have chars before the start of the match we
                // need to return them first.
                if no_match_end > 0 {
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
                        inner,
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
impl std::fmt::Display for Lexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.nfa.fmt(f)
    }
}

// TODO: QuickCheck or similar
#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::Lexer;
    use std::ops::Range;

    use super::*;

    #[test]
    fn lexer() {
        let nfa = Lexer::compile(&["let", "fn", "lefn", "letfn"]);

        assert_eq!(None, nfa.find_match("l"));
        assert_eq!(Some("let"), nfa.find_match("let"));
        assert_eq!(Some("fn"), nfa.find_match("fn"));
        assert_eq!(Some("letfn"), nfa.find_match("letfn"));
        assert_eq!(Some("lefn"), nfa.find_match("lefn"));
        assert_eq!(Some("fn"), nfa.find_match("fnlet"));
        assert_eq!(Some("letfn"), nfa.find_match("letfnn"));
        assert_eq!(Some("let"), nfa.find_match("letffn"));
    }

    #[test]
    fn longer() {
        let symbols = &[
            "struct",
            "fntype",
            "fn",
            "type",
            "enum",
            "letfn",
            "let",
            "structure",
        ];
        let lexer = Lexer::compile(symbols);
        let input = "".split("struct let fn type enum letfn fntype structure structurize fnlet typingenum hello world").collect::<Vec<_>>();

        for input in input {
            if let Some(a) = lexer.find_match(input) {
                assert!(symbols.contains(&a));
            } else {
                assert!(!symbols.contains(&input));
            }
        }
    }

    fn test_lexer(tokens: &[&str], source: &str, expected: &[(Range<usize>, &str)]) {
        let offset = rand::thread_rng().gen_range(0..100_000);

        println!("Lexing:\n\t'{source}'\nWith tokens:\n\t{tokens:?}");

        let lexer = Lexer::compile(tokens.into_iter().map(|s| *s));
        let mut scanner = lexer.scan(offset, source);

        for (span, inner) in expected.into_iter().cloned() {
            assert_eq!(
                scanner.next(),
                Some(Spanned {
                    span: offset + span.start..offset + span.end,
                    inner
                })
            );
        }

        assert!(scanner.next().is_none());
    }

    #[test]
    fn simple_tokens() {
        test_lexer(
            &["a", "b", "ab"],
            "a ab aa aabb  \n b",
            &[
                (0..1, "a"),
                (2..4, "ab"),
                (5..6, "a"),
                (6..7, "a"),
                (8..9, "a"),
                (9..11, "ab"),
                (11..12, "b"),
                (16..17, "b"),
            ],
        );
    }

    #[test]
    fn longer_tokens() {
        test_lexer(
            &["aa", "bb", "aabbcc", "c"],
            "\n\n\t   abc abb cc babaabbccaabbccc   \n",
            &[
                (6..8, "ab"),
                (8..9, "c"),
                (10..11, "a"),
                (11..13, "bb"),
                (14..15, "c"),
                (15..16, "c"),
                (17..20, "bab"),
                (20..26, "aabbcc"),
                (26..32, "aabbcc"),
                (32..33, "c"),
            ],
        );
    }
}
