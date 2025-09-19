use lexer::{Lexer, Scanner};

const DEFAULT_OPS: [&str; 6] = ["(", ")", ",", "{", "}", ";"];

pub struct ExprLexer {
    lexer: Lexer,
}

impl ExprLexer {
    pub fn new<'o, OI: IntoIterator<Item = &'o str>>(operators: OI) -> Self
    where
        OI: std::fmt::Debug,
    {
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
