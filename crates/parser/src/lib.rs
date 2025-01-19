use japp_util::Spanned;
use lexer::ExprLexer;
use nom::Finish;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

mod ast;
mod expr_parser;
mod lexer;
mod parser;

use ast::{Decl, Fixity, Program, UnparsedDecl};
use expr_parser::ExprParser;
use parser::parse_program;

pub use parser::{ErrorKind, ParseError};

pub fn parse(source: &str) -> Result<Program, Vec<ParseError>> {
    let (_, program) = parse_program(source).finish().map_err(|e| vec![e])?;

    let mut declarations = vec![];
    let mut operators: HashMap<&str, (Range<usize>, Fixity)> = HashMap::new();
    let mut errors = vec![];

    for decl in program.decls {
        match decl {
            UnparsedDecl::Infix { ident, fixity } => {
                let this_span = ident.byte_offset()..ident.len();
                if let Some((other_span, _)) =
                    operators.insert(ident.data(), (this_span.clone(), fixity))
                {
                    errors.push(ParseError {
                        span: this_span,
                        error: ErrorKind::DuplicateFixity { other: other_span },
                    });
                }
            }
            d => {
                declarations.push(d);
            }
        }
    }

    let lexer = ExprLexer::new(operators.iter().map(|a| *a.0));
    let mut tokens = HashSet::new();
    let mut parser = ExprParser::new(operators);
    let mut parsed_program = Program {
        declarations: vec![],
    };

    for decl in declarations {
        match decl {
            UnparsedDecl::Let { ident, rhs } => {
                let tokens = lexer.scan(rhs.byte_offset(), rhs.data());
                let expr = parser.parse(tokens).map_err(|e| vec![e])?;

                parsed_program.declarations.push(Decl::Let {
                    ident: ident.data(),
                    rhs: expr,
                });
            }
            UnparsedDecl::Fn { ident, args, body } => {
                let body_tokens = lexer.scan(body.byte_offset(), body.data());
                let body = parser.parse(body_tokens).map_err(|e| vec![e])?;

                tokens.insert(*ident.data());

                parsed_program.declarations.push(Decl::Fn {
                    ident: ident.data(),
                    args: args
                        .into_iter()
                        .map(|arg| Spanned {
                            span: arg.byte_offset()..arg.data().len(),
                            inner: *arg.data(),
                        })
                        .collect(),
                    body,
                });
            }
            UnparsedDecl::Infix { .. } => unreachable!(),
        }
    }

    if errors.is_empty() {
        Ok(parsed_program)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn t_parse_big() {
        let source = r#"
            infix + 2 ;
            infix * 2 ;

            let a = a ;
            let b = a + b ;

            fn add x y = x + y ;

            let zz = add(a * b, b) ;
        "#;
        let ast = parse(source).unwrap();

        assert_eq!(
            ast.to_string(),
            vec![
                "let a = a ;",
                "let b = ( a + b ) ;",
                "fn add x y = ( x + y ) ;",
                "let zz = add ( ( a * b ) , b ) ;",
            ]
            .join("\n")
        );
    }
}
