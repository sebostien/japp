pub struct ExprLexer {
    lexer: lexer::Lexer,
}

impl<S: AsRef<str>> FromIterator<S> for ExprLexer {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        let tokens = lexer::Lexer::compile(&iter.into_iter().collect::<Vec<_>>());

        Self { lexer: tokens }
    }
}

impl ExprLexer {
    pub fn tokenize(&self, source: &str) -> Vec<String> {
        let mut tokens = vec![];
        let source = source.chars().collect::<Vec<_>>();

        let mut current: Option<String> = None;
        let mut i = 0;
        while i < source.len() {
            if source[i].is_whitespace() {
                if let Some(token) = current.take() {
                    tokens.push(token);
                }
                i += 1;
            } else if let Some(m) = self
                .lexer
                .find(&source[i..].into_iter().collect::<String>())
            {
                if let Some(token) = current.take() {
                    tokens.push(token);
                }
                tokens.push(source[i..i + m].into_iter().collect());
                i += m;
            } else if let Some(prev) = current.as_mut() {
                prev.push(source[i]);
                i += 1;
            } else {
                current = Some(source[i].to_string());
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
    use super::ExprLexer;

    #[test]
    fn lexer() {
        let lexer = ExprLexer::from_iter(["a", "aa", "abb", "bb", "bab", "b"]);

        assert_eq!(
            lexer.tokenize("aa a abb bab aab bb abababbbaa"),
            ["aa", "a", "abb", "bab", "aa", "b", "bb", "a", "bab", "abb", "b", "aa"]
                .map(String::from)
                .to_vec()
        );
    }
}
