mod interleave;
mod nfa;
mod state;
mod token;

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
    pub fn find(&self, input: &str) -> Option<usize> {
        self.nfa.find(input)
    }

    #[must_use]
    pub fn find_match<'a>(&self, input: &'a str) -> Option<&'a str> {
        Some(&input[0..self.nfa.find(input)?])
    }
}

#[cfg(test)]
impl std::fmt::Display for Lexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.nfa.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::Lexer;

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
}
