use std::ops::Range;
use std::{collections::HashMap, iter::Peekable};

use crate::ast::{Assoc, Expr, Fixity, Lit, Spanned};

pub struct ExprParser<'ops> {
    operators: HashMap<&'ops str, (Range<usize>, Fixity)>,
}

impl<'ops> ExprParser<'ops> {
    pub fn new(operators: HashMap<&'ops str, (Range<usize>, Fixity)>) -> Self {
        Self { operators }
    }

    /// Parse an expression based on the operators in `self`.
    pub fn parse<'a, I: IntoIterator<Item = Spanned<&'a str>>>(
        &mut self,
        source: I,
    ) -> Result<Expr, String> {
        let mut source = source.into_iter().peekable();
        let expr = self.parse_expr(&mut source, 0)?;
        if let Some(token) = source.next() {
            Err(format!("Unexpected token '{:?}'", token))
        } else {
            Ok(expr)
        }
    }

    /// Recursively parse tokens.
    fn parse_expr<'a, I>(
        &mut self,
        tokens: &mut Peekable<I>,
        precedence: usize,
    ) -> Result<Expr, String>
    where
        I: Iterator<Item = Spanned<&'a str>>,
    {
        let mut lhs = self.primary(tokens)?;

        while let Some(op) = tokens.peek() {
            let op = op.inner;

            let fixity = match self.operators.get(op) {
                Some(fixity) => fixity,
                None => {
                    break;
                }
            };

            if fixity.1.prec < precedence {
                break;
            }

            if fixity.1.assoc == Assoc::None && fixity.1.prec == precedence {
                // Precedence for none associative operators must strictly increase
                break;
            }

            tokens.next();

            let next_prec = match fixity.1.assoc {
                Assoc::Left => fixity.1.prec + 1,
                Assoc::Right => fixity.1.prec,
                Assoc::None => fixity.1.prec + 1,
            };

            let rhs = self.parse_expr(tokens, next_prec)?;
            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op: op.to_string(),
                rhs: Box::new(rhs),
            }
        }

        Ok(lhs)
    }

    fn primary<'a, I>(&mut self, tokens: &mut Peekable<I>) -> Result<Expr, String>
    where
        I: Iterator<Item = Spanned<&'a str>>,
    {
        let token = tokens.next().unwrap();

        if token.inner == "(" {
            let expr = self.parse_expr(tokens, 0)?;
            if tokens.next().as_ref().map(Spanned::inner) != Some(&")") {
                return Err("Mismatched parens".to_string());
            }
            Ok(expr)
        } else {
            match Lit::from(token.inner()) {
                Lit::Ident(_) => {
                    if tokens.peek().map(Spanned::inner) == Some(&"(") {
                        return self.parse_f_call(tokens, token);
                    } else {
                        Ok(Expr::Lit(token.map(String::from).map(Lit::Ident)))
                    }
                }
                lit => Ok(Expr::Lit(Spanned::new(lit, token.span))),
            }
        }
    }

    fn parse_f_call<'a, I>(
        &mut self,
        tokens: &mut Peekable<I>,
        ident: Spanned<&str>,
    ) -> Result<Expr, String>
    where
        I: Iterator<Item = Spanned<&'a str>>,
    {
        let mut args = vec![];
        tokens.next(); // Consume '('

        while let Some(token) = tokens.peek().cloned() {
            if token.inner == ")" {
                tokens.next(); // Consume ')'
                return Ok(Expr::FCall {
                    ident: ident.inner.to_string(),
                    args,
                });
            } else if token.inner == "," {
                return Err(format!("Unexpected token ','"));
            }

            args.push(self.parse_expr(tokens, 0)?);
            if tokens.peek().map(Spanned::inner) == Some(&",") {
                tokens.next();
            }
        }

        Err("No matching ')' after function call".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::{Assoc, Expr, Fixity, Lit, Spanned};
    use crate::expr_parser::ExprParser;

    #[test]
    fn test_expr_parser() {
        let ops = HashMap::from_iter([
            (
                "+",
                (
                    0..0,
                    Fixity {
                        prec: 2,
                        assoc: Assoc::Left,
                    },
                ),
            ),
            (
                "-",
                (
                    0..0,
                    Fixity {
                        prec: 2,
                        assoc: Assoc::Left,
                    },
                ),
            ),
            (
                "*",
                (
                    0..0,
                    Fixity {
                        prec: 3,
                        assoc: Assoc::Left,
                    },
                ),
            ),
            (
                "/",
                (
                    0..0,
                    Fixity {
                        prec: 3,
                        assoc: Assoc::Left,
                    },
                ),
            ),
            (
                "^",
                (
                    0..0,
                    Fixity {
                        prec: 4,
                        assoc: Assoc::Right,
                    },
                ),
            ),
            (
                "==",
                (
                    0..0,
                    Fixity {
                        prec: 1,
                        assoc: Assoc::None,
                    },
                ),
            ),
        ]);
        let lexer = crate::lexer::ExprLexer::from_iter(ops.keys().copied());

        let source = "(2*2+2*3^2/(18))+2^3^2*11+(2-3/3)==add(5638,1)-1";
        let tokens = lexer.tokenize(source);

        assert_eq!(
            Ok(Lit::Bool(true)),
            ExprParser::new(ops).parse(tokens).unwrap().eval()
        );
    }

    #[test]
    fn simple() {
        let ops = HashMap::from_iter([(
            "*",
            (
                0..0,
                Fixity {
                    prec: 3,
                    assoc: Assoc::Left,
                },
            ),
        )]);
        let lexer = crate::lexer::ExprLexer::from_iter(ops.keys().copied());

        let source = "add(2*2, 2)";
        let tokens = lexer.tokenize(source);

        assert_eq!(
            Expr::FCall {
                ident: "add".to_string(),
                args: vec![
                    Expr::Binary {
                        lhs: Box::new(Expr::Lit(Spanned {
                            span: 4..5,
                            inner: Lit::Num(2)
                        })),
                        op: "*".to_string(),
                        rhs: Box::new(Expr::Lit(Spanned {
                            span: 6..7,
                            inner: Lit::Num(2)
                        }))
                    },
                    Expr::Lit(Spanned {
                        span: 9..10,
                        inner: Lit::Num(2)
                    })
                ]
            },
            ExprParser::new(ops).parse(tokens).unwrap()
        );
    }

    #[test]
    fn extra_comma() {
        let lexer = crate::lexer::ExprLexer::from_iter([]);

        let source = "add(2,,3)";
        let tokens = lexer.tokenize(source);
        assert!(ExprParser::new(HashMap::default()).parse(tokens).is_err());

        let source = "add(2,3,)";
        let tokens = lexer.tokenize(source);
        assert!(ExprParser::new(HashMap::default()).parse(tokens).is_ok());
    }
}
