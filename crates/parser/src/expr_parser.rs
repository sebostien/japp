use japp_util::Spanned;
use log::debug;
use std::ops::Range;
use std::{collections::HashMap, iter::Peekable};

use crate::ast::{Associativity, Expr, Fixity, Ident, Lit};
use crate::{ErrorKind, MatchBody, ParseError, Pattern};

pub struct ExprParser<'ops> {
    operators: HashMap<&'ops str, (Range<usize>, Fixity)>,
}

pub type ParseResult<'source, E = Expr<'source>> = Result<E, ParseError<'source>>;

impl<'ops> ExprParser<'ops> {
    pub fn new(mut operators: HashMap<&'ops str, (Range<usize>, Fixity)>) -> Self {
        // TODO: Should probably not do this here
        operators.insert(
            "=",
            (
                0..0,
                Fixity {
                    prec: 0,
                    assoc: Associativity::Right,
                },
            ),
        );

        Self { operators }
    }

    /// Parse an expression based on the operators in `self`.
    pub fn parse<'source, I: IntoIterator<Item = Spanned<&'source str>>>(
        &mut self,
        source: I,
    ) -> ParseResult<'source> {
        let mut source = source.into_iter().peekable();
        let expr = self.parse_expr(&mut source, 0)?;

        if let Some(token) = source.next() {
            Err(ParseError {
                span: token.span,
                error: ErrorKind::UnexpectedToken {
                    found: token.inner,
                    expected: ";",
                },
            })
        } else {
            Ok(expr)
        }
    }

    /// Recursively parse tokens.
    fn parse_expr<'source, I>(
        &mut self,
        tokens: &mut Peekable<I>,
        precedence: usize,
    ) -> ParseResult<'source>
    where
        I: Iterator<Item = Spanned<&'source str>>,
    {
        let mut lhs = self.primary(tokens, precedence)?;

        while let Some(op) = tokens.peek() {
            debug!("parse_expr with '{op}'");
            let op = op.clone();
            let op = Ident::new(op.span, op.inner);

            let fixity = match self.operators.get(op.inner()) {
                Some(fixity) => fixity,
                None => {
                    break;
                }
            };

            if fixity.1.prec < precedence {
                break;
            }

            if fixity.1.assoc == Associativity::None && fixity.1.prec == precedence {
                // Precedence for none associative operators must strictly increase
                return Err(ParseError {
                    span: op.outer_span(),
                    error: ErrorKind::ExprParser {
                        error: "Could not parse non associative expression".to_string(),
                    },
                });
            }

            tokens.next();

            let next_prec = match fixity.1.assoc {
                Associativity::Left => fixity.1.prec + 1,
                Associativity::Right => fixity.1.prec,
                Associativity::None => fixity.1.prec,
            };

            let rhs = self.parse_expr(tokens, next_prec)?;
            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op: op.clone(),
                rhs: Box::new(rhs),
            }
        }

        Ok(lhs)
    }

    fn primary<'source, I>(
        &mut self,
        tokens: &mut Peekable<I>,
        _precedence: usize,
    ) -> ParseResult<'source>
    where
        I: Iterator<Item = Spanned<&'source str>>,
    {
        let token = tokens.next().unwrap();

        if token.inner == "(" {
            let expr = self.parse_expr(tokens, 0)?;
            if let Some(cparen) = tokens.next() {
                if cparen.inner != ")" {
                    return Err(ParseError {
                        span: cparen.span,
                        error: ErrorKind::UnexpectedToken {
                            found: cparen.inner,
                            expected: ")",
                        },
                    });
                }
            }
            Ok(expr)
        } else if token.inner == "{" {
            let mut inner_exprs = Vec::new();
            loop {
                let expr = self.parse_expr(tokens, 0)?;
                inner_exprs.push(expr);

                match tokens.peek().map(|t| *t.inner()) {
                    Some(";") => {
                        let _ = tokens.next();
                        continue;
                    }
                    Some("}") => {
                        let _ = tokens.next();
                        break;
                    }
                    None => {
                        return Err(ParseError {
                            span: token.span.clone(),
                            error: ErrorKind::Mismatched {
                                start: token,
                                expected: Some("}"),
                                extra_info: "",
                            },
                        });
                    }
                    _ => {
                        continue;
                    }
                }
            }

            let last = inner_exprs.pop().map(Box::new);

            Ok(Expr::Block {
                exprs: inner_exprs,
                last,
            })
        } else if token.inner == "match" {
            let var = self.parse_expr(tokens, 0)?;
            let body = self.parse_match_block(tokens)?;

            Ok(Expr::Match {
                var: Box::new(var),
                body,
            })
        } else if let Some((_, fix)) = self.operators.get(token.inner) {
            // TODO: Precedence
            let rhs = self.parse_expr(tokens, fix.prec)?;

            Ok(Expr::Prefix {
                op: Ident::new(token.span, token.inner),
                rhs: Box::new(rhs),
            })
        } else {
            let span = token.span.clone();
            let lit = Lit::from(token.clone());

            if let Lit::Ident(ref ident) = lit {
                if tokens.peek().map(Spanned::inner) == Some(&"(") {
                    return self.parse_f_call(tokens, ident.clone());
                }
            }

            Ok(Expr::Lit(Spanned::new(lit, span)))
        }
    }

    fn consume<'source, I>(
        &mut self,
        tokens: &mut Peekable<I>,
        expected: &'source str,
    ) -> ParseResult<'source, Spanned<&'source str>>
    where
        I: Iterator<Item = Spanned<&'source str>>,
    {
        if let Some(token) = tokens.next() {
            let span = token.span.clone();
            if token.inner == expected {
                Ok(token)
            } else {
                Err(ParseError {
                    span: span,
                    error: ErrorKind::UnexpectedToken {
                        found: token.inner,
                        expected,
                    },
                })
            }
        } else {
            Err(ParseError {
                span: 0..0,
                error: ErrorKind::UnexpectedEof { expected },
            })
        }
    }

    fn parse_f_call<'source, I>(
        &mut self,
        tokens: &mut Peekable<I>,
        ident: Ident<'source>,
    ) -> ParseResult<'source>
    where
        I: Iterator<Item = Spanned<&'source str>>,
    {
        let paren_start = tokens
            .next()
            .expect("`parse_f_call` was called without '(' as the next token"); // Consume '('

        let mut args = vec![];

        while let Some(token) = tokens.peek().cloned() {
            if token.inner == ")" {
                tokens.next(); // Consume ')'
                return Ok(Expr::FCall { ident, args });
            } else if token.inner == "," {
                return Err(ParseError {
                    span: token.span,
                    error: ErrorKind::UnexpectedToken {
                        found: token.inner,
                        expected: "Expression",
                    },
                });
            }

            args.push(self.parse_expr(tokens, 0)?);
            if tokens.peek().map(Spanned::inner) == Some(&",") {
                tokens.next();
            }
        }

        Err(ParseError {
            span: paren_start.span.clone(),
            error: ErrorKind::Mismatched {
                start: paren_start,
                expected: Some(")"),
                extra_info: "This '(' was not closed",
            },
        })
    }

    fn parse_match_block<'source, I>(
        &mut self,
        tokens: &mut Peekable<I>,
    ) -> ParseResult<'source, MatchBody<'source>>
    where
        I: Iterator<Item = Spanned<&'source str>>,
    {
        let open_brace = self.consume(tokens, "{")?;
        let mut cases = Vec::new();

        'outer: loop {
            let pattern = self.parse_pattern(tokens)?;
            let _arrow = self.consume(tokens, "->")?;
            let body = self.parse_expr(tokens, 0)?;

            cases.push((pattern, body));

            loop {
                match tokens.peek().map(|t| *t.inner()) {
                    Some(";") => {
                        let _ = tokens.next();
                        continue;
                    }
                    Some("}") => {
                        let _ = tokens.next();
                        break 'outer;
                    }
                    None => {
                        return Err(ParseError {
                            span: open_brace.span.clone(),
                            error: ErrorKind::Mismatched {
                                start: open_brace,
                                expected: Some("}"),
                                extra_info: "",
                            },
                        });
                    }
                    _ => {
                        break;
                    }
                }
            }
        }

        Ok(MatchBody { cases })
    }

    fn parse_pattern<'source, I>(
        &mut self,
        tokens: &mut Peekable<I>,
    ) -> ParseResult<'source, Pattern<'source>>
    where
        I: Iterator<Item = Spanned<&'source str>>,
    {
        Ok(Pattern::Lit(Lit::from(tokens.next().unwrap())))
    }
}

#[cfg(test)]
mod tests {
    use japp_util::Spanned;
    use std::collections::HashMap;
    use std::ops::Range;

    use crate::ast::{Associativity, Expr, Fixity, Ident, Lit};
    use crate::expr_parser::ExprParser;
    use crate::lexer::ExprLexer;

    fn get_test_ops() -> HashMap<&'static str, (Range<usize>, Fixity)> {
        [
            ("+", 2, Associativity::Left),
            ("-", 2, Associativity::Left),
            ("*", 3, Associativity::Left),
            ("/", 3, Associativity::Left),
            ("^", 4, Associativity::Right),
            ("==", 1, Associativity::None),
            ("=", 0, Associativity::Right),
        ]
        .into_iter()
        .map(|(op, prec, assoc)| (op, (0..0, Fixity { prec, assoc })))
        .collect()
    }

    #[test]
    fn test_expr_parser() {
        let ops = get_test_ops();
        let lexer = ExprLexer::new(ops.keys().copied());

        let source = "(2*2+2*3^2/(18))+2^3^2*11+(2-3/3)==add(5638,1)-1";
        let tokens = lexer.scan(0, source);

        assert_eq!(
            Ok(Lit::Bool(true)),
            ExprParser::new(ops).parse(tokens).unwrap().eval()
        );
    }

    #[test]
    fn simple() {
        let ops = get_test_ops();
        let lexer = ExprLexer::new(ops.keys().copied());

        let source = "add(2*2, 2)";
        let tokens = lexer.scan(0, source);

        assert_eq!(
            Expr::FCall {
                ident: Ident::new(0..3, "add"),
                args: vec![
                    Expr::Binary {
                        lhs: Box::new(Expr::Lit(Spanned {
                            span: 4..5,
                            inner: Lit::Int(2)
                        })),
                        op: Ident::new(5..6, "*"),
                        rhs: Box::new(Expr::Lit(Spanned {
                            span: 6..7,
                            inner: Lit::Int(2)
                        }))
                    },
                    Expr::Lit(Spanned {
                        span: 9..10,
                        inner: Lit::Int(2)
                    })
                ]
            },
            ExprParser::new(ops).parse(tokens).unwrap()
        );
    }

    #[test]
    fn extra_comma() {
        let lexer = ExprLexer::new([]);

        let source = "add(2,,3)";
        let tokens = lexer.scan(0, source);
        assert!(ExprParser::new(HashMap::default()).parse(tokens).is_err());

        let source = "add(2,3,)";
        let tokens = lexer.scan(0, source);
        assert!(ExprParser::new(HashMap::default()).parse(tokens).is_ok());
    }

    #[test]
    fn block() {
        let lexer = ExprLexer::new([]);

        let source = "{ 1 }";
        let tokens = lexer.scan(0, source);
        if let Err(e) = ExprParser::new(HashMap::default()).parse(tokens) {
            eprintln!("{e:?}");
            panic!("Expected ok");
        }

        let source = "{ println(2, 4, 3) ; 1337 }";
        let tokens = lexer.scan(0, source);
        if let Err(e) = ExprParser::new(HashMap::default()).parse(tokens) {
            eprintln!("{e:?}");
            panic!("Expected ok");
        }
    }

    #[test]
    fn prefix() {
        let ops = [(
            "!",
            (
                0..0,
                Fixity {
                    prec: 1,
                    assoc: Associativity::Right,
                },
            ),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let lexer = ExprLexer::new(ops.keys().copied());

        let source = "! 1";
        let tokens = lexer.scan(0, source);
        let result = ExprParser::new(ops.clone()).parse(tokens).unwrap();
        assert_eq!(
            result,
            Expr::Prefix {
                op: Ident::new(0..1, "!"),
                rhs: Box::new(Expr::Lit(Spanned::new(Lit::Int(1), 2..3))),
            }
        );

        let source = "!!!!5";
        let tokens = lexer.scan(0, source);
        if let Err(e) = ExprParser::new(ops).parse(tokens) {
            eprintln!("{e:?}");
            panic!("Expected ok");
        }
    }

    #[test]
    fn assign() {
        let ops = get_test_ops();
        let lexer = ExprLexer::new(ops.keys().copied());

        let source = "x = 2";
        let tokens = lexer.scan(0, source);
        match ExprParser::new(ops.clone()).parse(tokens) {
            Err(e) => {
                eprintln!("{e:?}");
                panic!("Expected ok");
            }
            Ok(ast) => {
                assert_eq!(
                    ast,
                    Expr::Binary {
                        lhs: Box::new(Expr::Lit(Spanned::new(
                            Lit::Ident(Ident::new(0..1, "x")),
                            0..1
                        ))),
                        op: Ident::new(2..3, "="),
                        rhs: Box::new(Expr::Lit(Spanned::new(Lit::Int(2), 4..5))),
                    }
                );
            }
        }

        let source = "x = y = 3";
        let tokens = lexer.scan(0, source);
        if let Err(e) = ExprParser::new(ops).parse(tokens) {
            eprintln!("{e:?}");
            panic!("Expected ok");
        }
    }
}
