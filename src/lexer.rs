use crate::ast::Spanned;

const DEFAULT_OPS: [&'static str; 3] = ["(", ")", ","];

pub struct ExprLexer {
    lexer: lexer::Lexer,
}

impl<'a> FromIterator<&'a str> for ExprLexer {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let ops = iter.into_iter().chain(DEFAULT_OPS);
        let tokens = lexer::Lexer::compile(ops);

        Self { lexer: tokens }
    }
}

impl ExprLexer {
    pub fn tokenize<'a>(&self, source: &'a str) -> Vec<Spanned<&'a str>> {
        let mut tokens = vec![];
        let chars = source.chars().collect::<Vec<_>>();

        let mut current: Option<Spanned<&'a str>> = None;
        let mut i = 0;
        while i < chars.len() {
            if chars[i].is_whitespace() {
                if let Some(token) = current.take() {
                    tokens.push(token);
                }
                i += 1;
            } else if let Some(m) = self.lexer.find(&chars[i..].into_iter().collect::<String>()) {
                if let Some(token) = current.take() {
                    tokens.push(token);
                }
                tokens.push(Spanned {
                    span: i..i + m,
                    inner: &source[i..i + m],
                });
                i += m;
            } else if let Some(prev) = current.as_mut() {
                prev.span = prev.span.start..prev.span.end + 1;
                prev.inner = &source[prev.span.clone()];
                i += 1;
            } else {
                current = Some(Spanned {
                    span: i..i + 1,
                    inner: &source[i..i + 1],
                });
                i += 1;
            }
        }

        if let Some(current) = current {
            tokens.push(current);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Spanned;

    use super::ExprLexer;

    #[test]
    fn lexer() {
        let lexer = ExprLexer::from_iter(["a", "aa", "abb", "bb", "bab", "b"]);

        assert_eq!(
            lexer
                .tokenize("aa a abb bab aab bb abababbbaa")
                .into_iter()
                .map(Spanned::take_inner)
                .collect::<Vec<_>>(),
            ["aa", "a", "abb", "bab", "aa", "b", "bb", "a", "bab", "abb", "b", "aa"]
                .map(String::from)
                .to_vec()
        );
    }

    #[test]
    fn spans() {
        let lexer = ExprLexer::from_iter(["a", "b", "ab"]);

        assert_eq!(
            lexer.tokenize("aa ab b   b"),
            [
                Spanned {
                    span: 0..1,
                    inner: "a",
                },
                Spanned {
                    span: 1..2,
                    inner: "a"
                },
                Spanned {
                    span: 3..5,
                    inner: "ab"
                },
                Spanned {
                    span: 6..7,
                    inner: "b"
                },
                Spanned {
                    span: 10..11,
                    inner: "b"
                },
            ],
        );
    }
}
