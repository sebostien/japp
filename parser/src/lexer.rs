use lexer::Lexer;
use std::str::Chars;

use crate::ast::Spanned;

const DEFAULT_OPS: [&'static str; 3] = ["(", ")", ","];

pub struct ExprLexer {
    lexer: Lexer,
}

impl ExprLexer {
    pub fn new<'o, OI: IntoIterator<Item = &'o str>>(operators: OI) -> Self {
        let ops = operators.into_iter().chain(DEFAULT_OPS);
        let lexer = Lexer::compile(ops);

        Self { lexer }
    }
}

impl ExprLexer {
    pub fn get_tokenizer<'l, 'source>(
        &'l self,
        offset: usize,
        source: &'source str,
    ) -> Tokenizer<'l, 'source> {
        Tokenizer {
            lexer: &self.lexer,
            source,
            chars: source.chars(),
            next: None,
            offset,
        }
    }
}

pub struct Tokenizer<'l, 'source> {
    lexer: &'l Lexer,
    source: &'source str,
    chars: Chars<'source>,
    next: Option<Spanned<&'source str>>,
    offset: usize,
}

impl<'source> Iterator for Tokenizer<'_, 'source> {
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

// TODO: QuickCheck or similar
#[cfg(test)]
mod tests {
    use std::ops::Range;

    use rand::Rng;

    use crate::ast::Spanned;

    use super::ExprLexer;

    fn test_lexer(tokens: &[&str], source: &str, expected: &[(Range<usize>, &str)]) {
        println!("Lexing:\n\t'{source}'\nWith tokens:\n\t{tokens:?}");

        let offset = rand::thread_rng().gen_range(0..100_000);

        let lexer = ExprLexer::new(tokens.into_iter().map(|s| *s));
        let mut tokenizer = lexer.get_tokenizer(offset, source);

        for (span, inner) in expected {
            assert_eq!(
                tokenizer.next(),
                Some(Spanned {
                    span: offset + span.start..offset + span.end,
                    inner: *inner
                })
            );
        }

        assert!(tokenizer.next().is_none());
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
