use japp_util::Spanned;
use lexer::ExprLexer;
use nom::Finish;
use std::collections::HashMap;
use std::ops::Range;

// TODO: Remove Binary and Prefix and make them use FCall instead

mod ast;
mod error;
mod expr_parser;
mod lexer;
mod parser;

pub use ast::{Decl, Expr, Fixity, Ident, Lit, MatchBody, Pattern, Program, Type};
pub use error::{ErrorKind, ParseError};

use ast::UnparsedDecl;
use expr_parser::ExprParser;
use parser::parse_program;

pub fn parse(source: &str) -> Result<Program<'_>, Vec<ParseError<'_>>> {
    let (_, program) = parse_program(source).finish().map_err(|e| vec![e])?;

    let mut declarations = vec![];
    let mut operators: HashMap<&str, (Range<usize>, Fixity)> = HashMap::new();
    let mut errors = vec![];

    for decl in program.decls {
        match decl {
            UnparsedDecl::Infix { ident, fixity } => {
                let this_span = ident.outer_span();
                if let Some((other_span, _)) =
                    operators.insert(ident.inner(), (this_span.clone(), fixity))
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
    let mut parser = ExprParser::new(operators);
    let mut parsed_program = Program {
        declarations: HashMap::new(),
    };

    for decl in declarations {
        match decl {
            UnparsedDecl::Const { ident, rhs } => {
                let tokens = lexer.scan(rhs.byte_offset(), rhs.data());
                let expr = parser.parse(tokens).map_err(|e| vec![e])?;

                parsed_program
                    .declarations
                    .insert(ident.inner(), Decl::Const { ident, rhs: expr });
            }
            UnparsedDecl::Fn { ident, args, body } => {
                let body_tokens = lexer.scan(body.byte_offset(), body.data());
                let body_parsed = parser.parse(body_tokens).map_err(|e| vec![e])?;
                let fn_parsed = Decl::Fn {
                    ident: ident.clone(),
                    type_def: None,
                    args,
                    body: body_parsed,
                };

                if let Some(prev) = parsed_program.declarations.insert(ident.inner(), fn_parsed) {
                    if let Decl::Fn { body, .. } = prev {
                        if body != Expr::Lit(Spanned::new(Lit::Null, 0..0)) {
                            todo!("Multiple definitions for function {}", ident.inner())
                        }
                    }
                }
            }
            UnparsedDecl::FnSig { ident, sig } => {
                let prev = parsed_program
                    .declarations
                    .entry(ident.inner())
                    .or_insert(Decl::Fn {
                        ident,
                        type_def: None,
                        args: vec![],
                        body: Expr::Lit(Spanned::new(Lit::Null, 0..0)),
                    });

                match prev {
                    Decl::Const { .. } => panic!(),
                    Decl::Fn {
                        ident,
                        ref mut type_def,
                        ..
                    } => {
                        if type_def.is_some() {
                            todo!("Fn sig already defined for {}", ident.inner())
                        } else {
                            let _ = type_def.insert(sig);
                        }
                    }
                }
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
    use ariadne::Source;

    use crate::parse;

    #[test]
    fn t_parse_big() {
        let source = r#"
            infix + 2 ;
            infix * 2 ;

            const a = a ;
            const b = a + b ;

            fn add x y = x + y ;

            const zz = add(a * b, b) ;
        "#;
        let ast = parse(source).unwrap();

        assert_eq!(
            ast.to_string(),
            vec![
                "const a = a ;",
                "fn add x y = ( x + y ) ;",
                "const b = ( a + b ) ;",
                "const zz = add ( ( a * b ) , b ) ;",
            ]
            .join("\n")
        );
    }

    #[test]
    fn t_pattern_match() {
        let source = r#"
            infix - 1 ;
            infix * 2 ;

            fn fac n = match n {
                0 -> 1               ;
                n -> n * fac (n - 1) ;
            } ;

            fn main = fac(5) ;
        "#;
        let ast = parse(source).unwrap();

        assert_eq!(
            ast.to_string(),
            vec![
                "fn fac n = match n { 0 -> 1 ; n -> ( n * fac ( ( n - 1 ) ) ) ; } ;",
                "fn main = fac ( 5 ) ;",
            ]
            .join("\n")
        );
    }

    #[test]
    fn t_func_block_body() {
        let source = r#"
            fn fac = { 1 ; 2 ; 3 } ;
        "#;
        let ast = parse(source)
            .map_err(|e| {
                e[0].make_report("test")
                    .eprint(("test", Source::from(source)))
            })
            .unwrap();

        assert_eq!(ast.to_string(), vec!["fn fac = { 1 ; 2 ; 3 } ;"].join("\n"));
    }
}
