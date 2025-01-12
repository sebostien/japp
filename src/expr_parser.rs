use std::collections::HashMap;

use crate::ast::{Assoc, Expr, Fixity, Lit, Spanned};

pub struct ExprParser {
    pos: usize,
    operators: HashMap<String, Fixity>,
}

impl ExprParser {
    pub fn new(operators: HashMap<String, Fixity>) -> Self {
        Self { pos: 0, operators }
    }

    /// Parse an expression based on the operators in `self`.
    pub fn parse(&mut self, source: Vec<Spanned<String>>) -> Result<Expr, String> {
        self.pos = 0;
        let expr = self.parse_expr(&source, 0)?;
        if self.pos != source.len() {
            Err(format!("Unexpected token '{:?}'", source[self.pos]))
        } else {
            Ok(expr)
        }
    }

    /// Recursive parsing of expressions based on precedence.
    fn parse_expr(
        &mut self,
        tokens: &Vec<Spanned<String>>,
        precedence: usize,
    ) -> Result<Expr, String> {
        let mut lhs = self.primary(tokens)?;

        while self.pos < tokens.len() {
            let op = tokens[self.pos].inner.clone();
            let fixity = match self.operators.get(&op) {
                Some(fixity) => fixity,
                None => {
                    break;
                }
            };

            if fixity.prec < precedence {
                break;
            }

            if fixity.assoc == Assoc::None && fixity.prec == precedence {
                // Precedence for none associative operators must strictly increase
                break;
            }

            self.pos += 1;

            let next_prec = match fixity.assoc {
                Assoc::Left => fixity.prec + 1,
                Assoc::Right => fixity.prec,
                Assoc::None => fixity.prec + 1,
            };

            let rhs = self.parse_expr(tokens, next_prec)?;
            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        }

        Ok(lhs)
    }

    fn primary(&mut self, tokens: &Vec<Spanned<String>>) -> Result<Expr, String> {
        let token = tokens[self.pos].clone();
        self.pos += 1;

        if &token.inner == "(" {
            let expr = self.parse_expr(tokens, 0)?;
            if tokens[self.pos].inner != ")" {
                return Err("Mismatched parens".to_string());
            }
            self.pos += 1; // Consume ")"
            Ok(expr)
        } else {
            match Lit::from(&token.inner) {
                Lit::Ident(i) => {
                    if self.pos < tokens.len() && tokens[self.pos].inner == "(" {
                        return self.parse_f_call(tokens, token);
                    } else {
                        Ok(Expr::Lit(Lit::Ident(i)))
                    }
                }
                lit => Ok(Expr::Lit(lit)),
            }
        }
    }

    fn parse_f_call(
        &mut self,
        tokens: &Vec<Spanned<String>>,
        ident: Spanned<String>,
    ) -> Result<Expr, String> {
        let mut args = vec![];
        self.pos += 1; // Consume '('

        while self.pos < tokens.len() {
            if tokens[self.pos].inner == ")" {
                self.pos += 1; // Consume ')'
                return Ok(Expr::FCall {
                    ident: ident.inner,
                    args,
                });
            }

            args.push(self.parse_expr(tokens, 0)?);
            if tokens[self.pos].inner == "," {
                self.pos += 1;
            }
        }

        Err("No matching ')' after function call".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::{Assoc, Fixity, Lit};
    use crate::expr_parser::ExprParser;

    #[test]
    fn test_expr_parser() {
        let ops = HashMap::<String, Fixity>::from_iter([
            (
                "+".to_string(),
                Fixity {
                    prec: 2,
                    assoc: Assoc::Left,
                },
            ),
            (
                "-".to_string(),
                Fixity {
                    prec: 2,
                    assoc: Assoc::Left,
                },
            ),
            (
                "*".to_string(),
                Fixity {
                    prec: 3,
                    assoc: Assoc::Left,
                },
            ),
            (
                "/".to_string(),
                Fixity {
                    prec: 3,
                    assoc: Assoc::Left,
                },
            ),
            (
                "^".to_string(),
                Fixity {
                    prec: 4,
                    assoc: Assoc::Right,
                },
            ),
            (
                "==".to_string(),
                Fixity {
                    prec: 1,
                    assoc: Assoc::None,
                },
            ),
        ]);
        let tokens = "(2*2+2*3^2/(18))+2^3^2*11+(2-3/3)==add(5638,1)-1";
        let lexer =
            crate::lexer::ExprLexer::from_iter(["+", "-", "*", "/", "^", "==", "(", ")", ","]);
        let tokens = lexer.tokenize(tokens);

        assert_eq!(
            Lit::Bool(true),
            ExprParser::new(ops).parse(tokens).unwrap().eval()
        );
    }
}
