use lexer::{Lexer, Scanner};

const DEFAULT_OPS: [&str; 3] = ["(", ")", ","];

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
    pub fn scan<'l, 'source>(
        &'l self,
        offset: usize,
        source: &'source str,
    ) -> Scanner<'l, 'source> {
        self.lexer.scan(offset, source)
    }
}

// TODO: QuickCheck or similar
#[cfg(test)]
mod tests {
    use japp_util::Spanned;
    use std::ops::Range;

    use rand::Rng;

    use super::ExprLexer;

    fn test_lexer(tokens: &[&str], source: &str, expected: &[(Range<usize>, &str)]) {
        println!("Lexing:\n\t'{source}'\nWith tokens:\n\t{tokens:?}");

        let offset = rand::thread_rng().gen_range(0..100_000);

        let lexer = ExprLexer::new(tokens.into_iter().map(|s| *s));
        let mut tokenizer = lexer.scan(offset, source);

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
