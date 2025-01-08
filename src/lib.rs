use ariadne::{Label, Report, ReportKind};
use chumsky::{error::SimpleReason, prelude::*};
use std::ops::Range;

mod decl;
mod program;

use decl::{Assoc, Decl, Spanned};
use program::Program;
use text::TextParser;

pub fn make_reports<'f>(
    file_name: &'f str,
    errors: &[Simple<char>],
) -> Vec<Report<'f, (&'f str, Range<usize>)>> {
    errors
        .iter()
        .map(|error| {
            let mut report = Report::build(ReportKind::Error, (file_name, error.span()));

            let message = match error.reason() {
                SimpleReason::Unexpected => {
                    report = report.with_message("Unexpected token");
                    error.to_string()
                }
                SimpleReason::Unclosed { span, delimiter } => todo!(
                    "Check if the following is a good message: {:?} {} ::: {:?} ::: {}",
                    error,
                    error,
                    span,
                    delimiter
                ),
                SimpleReason::Custom(e) => e.clone(),
            };

            report
                .with_label(Label::new((file_name, error.span())).with_message(message))
                .finish()
        })
        .collect()
}

const DEFAULT_PREC: usize = 10;

/// Any utf-8 (without whitespace) sequence not starting with an ascii digit.
fn ident() -> impl Parser<char, Spanned<String>, Error = Simple<char>> {
    filter(|&c: &char| c != ';')
        .then(filter(|c: &char| !c.is_whitespace()).repeated())
        .padded()
        .map(|(c, mut cs)| {
            cs.insert(0, c);
            cs.iter().collect()
        })
        .try_map(|inner: String, span| {
            if matches!(inner.as_str(), "=" | ";") {
                Err(Simple::custom(span, format!("Invalid ident '{inner}'")))
            } else {
                Ok(Spanned { span, inner })
            }
        })
}

fn unparsed_expr() -> impl Parser<char, Spanned<String>, Error = Simple<char>> {
    filter(|&c: &char| c != ';')
        .repeated()
        .at_least(1)
        .padded()
        // TODO: Padded only trims before, we manually trim the end.
        //       Find way to avoid this
        .map_with_span(|cs, span| Spanned {
            span,
            inner: cs.into_iter().collect::<String>().trim().to_owned(),
        })
}

fn decl() -> impl Parser<char, Decl, Error = Simple<char>> {
    let infix_decl = choice((
        text::keyword("infixl").map(|_| Assoc::Left),
        text::keyword("infixr").map(|_| Assoc::Right),
        text::keyword("infix").map(|_| Assoc::None),
    ))
    .then(ident())
    .then(chumsky::text::int(10).validate(|x: String, span, emit| {
        if let Ok(n) = x.parse() {
            n
        } else {
            emit(Simple::custom(
                span,
                format!("Max precedence is {}", usize::MAX),
            ));
            DEFAULT_PREC
        }
    }))
    .then_ignore(just(';'))
    .map(|((assoc, ident), prec)| Decl::Infix { ident, prec, assoc });

    let let_decl = text::keyword("let")
        .ignore_then(ident())
        .then_ignore(just('='))
        .then(unparsed_expr())
        .then_ignore(just(';'))
        .map(|(ident, rhs)| Decl::Let { ident, rhs });

    let fn_decl = text::keyword("fn")
        .ignore_then(ident())
        .then(ident().repeated())
        .then_ignore(just('='))
        .then(unparsed_expr())
        .then_ignore(just(';'))
        .map(|((ident, args), body)| Decl::Fn { ident, args, body });

    infix_decl.or(let_decl).or(fn_decl).padded()
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    decl()
        .repeated()
        .map(|exprs| Program { exprs })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser_ok(test_loc: &str, source: &str, expected: &str) {
        let file_name = "test";

        match parser().parse(source) {
            Ok(actual) => {
                assert_eq!(
                    actual.to_string(),
                    expected,
                    "Not the expected result from test {test_loc}\nParsing:\n'''\n{source}\n'''\nActual (left) vs Expected (right)"
                );
            }
            Err(e) => {
                let mut out = Vec::new();
                for report in make_reports(file_name, &e) {
                    report
                        .write((file_name, ariadne::Source::from(source)), &mut out)
                        .unwrap();
                }
                let out = String::from_utf8(out).unwrap();
                panic!("Expected successful parse on test {test_loc}\n{out}");
            }
        }
    }

    macro_rules! test_parser_ok {
        ($source:expr,$expected:expr,) => {
            test_parser_ok!($source, $expected)
        };
        ($source:expr,$expected:expr) => {
            test_parser_ok(
                &format!("{}:{}:{}", file!(), line!(), column!()),
                $source,
                $expected,
            );
        };
    }

    #[test]
    fn t_parse_let() {
        test_parser_ok!("let a = -   -a;", "let a = -   -a ;");
        test_parser_ok!("\nlet \n c = \n -\n\n   a \n;", "let c = -\n\n   a ;");
        test_parser_ok!("let z =  -a  \n\n; ", "let z = -a ;");
        test_parser_ok!("let a23 = a + b ;", "let a23 = a + b ;");
        test_parser_ok!(
            "let add = 1+(2 + (3+((4)))) == (1 + 2 + 3 +4);",
            "let add = 1+(2 + (3+((4)))) == (1 + 2 + 3 +4) ;"
        );
        test_parser_ok!(
            "let abc = a + b * c /(2/d) ;",
            "let abc = a + b * c /(2/d) ;"
        );
    }

    #[test]
    fn t_fn() {
        test_parser_ok!("fn add x y = x + y; ", "fn add x y = x + y ;");
    }

    #[test]
    fn t_partial_application() {
        test_parser_ok!(
            "fn add x y = x + y; let z = add(x)(y);",
            "fn add x y = x + y ;\nlet z = add(x)(y) ;"
        );
    }

    #[test]
    fn t_infix() {
        test_parser_ok!("infix asdkj23lka9* 10;", "infix asdkj23lka9* 10;");
        test_parser_ok!("infixl jasdk 10;", "infixl jasdk 10;");
        test_parser_ok!("infixr asld 10;", "infixr asld 10;");
        test_parser_ok!("infixr >=> 0;", "infixr >=> 0;");
    }

    #[test]
    fn t_idents() {
        test_parser_ok!(
            "fn . 8 -9123 == = <=> < + -;  ",
            "fn . 8 -9123 == = <=> < + - ;"
        );
    }

    #[test]
    fn t_parse_big() {
        test_parser_ok!(
            r#"
                let a = a ;
                let b = a + b ;

                fn add x y = x + y ;

                let zz = add(a * b, b) ;

            "#,
            &vec![
                "let a = a ;",
                "let b = a + b ;",
                "fn add x y = x + y ;",
                "let zz = add(a * b, b) ;",
            ]
            .join("\n"),
        );
    }

    //////////////////////////////////////////////////
    ///// Errors ////////////////////////////////////
    ////////////////////////////////////////////////

    fn test_parser_err(test_loc: &str, source: &str, reason: &str) {
        match parser().parse(source) {
            Ok(actual) => {
                panic!(
                    "Parser successfully parsed test {}:{test_loc}\nWith result:\n'''\n{actual}\n'''\nBut this should fail because: {reason}",
                    file!()
                );
            }
            Err(_) => {
                // TODO: Validate msg
            }
        }
    }

    macro_rules! test_parser_err {
        ($source:expr,$reason:expr,) => {
            test_parser_err!($source, $reason)
        };
        ($source:expr,$reason:expr) => {
            test_parser_err(
                &format!("{}:{}:{}", file!(), line!(), column!()),
                $source,
                $reason,
            );
        };
    }

    #[test]
    fn t_error_no_semi() {
        test_parser_err!("let z = 2", "No semi at end of decl");
        test_parser_err!("let z = 2; let b = 2", "No semi at end of decl");
        test_parser_err!("fn z = 2; let b = 2", "No semi at end of decl");
        test_parser_err!("let b = 2 ; fn z = 2 \n\n", "No semi at end of decl");
    }

    #[test]
    fn t_error_prec_to_big() {
        test_parser_err!(
            "infixr a 1231293810293812903890128390183209813821039 ;",
            "Precedence too big. Must fit in usize"
        );
    }
}
