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
    use crate::ast::Spanned;

    use super::ExprLexer;

    #[test]
    fn simple() {
        let lexer = ExprLexer::new(["!", "!!"]);
        let tokenizer = lexer.get_tokenizer(0, "! !! !!! ! !! ! ! !!");

        assert_eq!(
            tokenizer.map(Spanned::take_inner).collect::<Vec<_>>(),
            ["!", "!!", "!!", "!", "!", "!!", "!", "!", "!!"].to_vec()
        );
    }

    #[test]
    fn spans() {
        let lexer = ExprLexer::new(["a", "b", "ab"]);
        let mut tokenizer = lexer.get_tokenizer(0, "a ab aa aabb   b");

        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 0..1,
                inner: "a"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 2..4,
                inner: "ab"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 5..6,
                inner: "a"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 6..7,
                inner: "a"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 8..9,
                inner: "a"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 9..11,
                inner: "ab"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 11..12,
                inner: "b"
            })
        );
        assert_eq!(
            tokenizer.next(),
            Some(Spanned {
                span: 15..16,
                inner: "b"
            })
        );
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn other() {
        let lexer = ExprLexer::new(["aa", "bb", "aabbcc", "c"]);
        let mut tokenizer = lexer.get_tokenizer(0, "abc abb cc babaabbccaabbccc ");

        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("ab"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("c"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("a"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("bb"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("c"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("c"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("bab"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("aabbcc"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("aabbcc"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), Some("c"));
        assert_eq!(tokenizer.next().map(Spanned::take_inner), None);
        assert_eq!(tokenizer.next().map(Spanned::take_inner), None);
        assert_eq!(tokenizer.next().map(Spanned::take_inner), None);
    }
}
