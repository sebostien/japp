use std::ops::Range;

///
/// Program = Decl*
///
/// Decl    = Type | Struct | Func
///
/// Type    = "type" Ident = Ty
///
/// Struct  = "struct" Ident
///
/// Ty
use ariadne::{Label, Report, ReportKind};
use chumsky::prelude::*;

mod expr;

use expr::{BinOp, Expr, Program};
use text::TextParser;

pub fn make_reports(
    file_name: &str,
    errors: Vec<Simple<char>>,
) -> Vec<Report<(&str, Range<usize>)>> {
    errors
        .iter()
        .map(|error| {
            // eprintln!("{error:?}");
            Report::build(ReportKind::Error, (file_name, error.span()))
                .with_label(Label::new((file_name, error.span())).with_message(error.to_string()))
                .finish()
        })
        .collect()
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let ident = text::ident().padded();

    let expr = recursive(|expr| {
        let call = ident
            .clone()
            .then(
                expr.clone()
                    .separated_by(just(','))
                    .allow_trailing() // Foo is Rust-like, so allow trailing commas to appear in arg lists
                    .delimited_by(just('('), just(')')),
            )
            .map(|(f, args)| Expr::Call(Box::new(Expr::Ident(f)), args));

        let atom = expr
            .delimited_by(just('('), just(')'))
            .or(call)
            .or(ident.map(Expr::Ident));

        let op = |c| just(c).padded();

        let unary = op('-')
            .clone()
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        let product = unary
            .clone()
            .then(one_of("*/").then(unary).repeated())
            .foldl(|lhs, (op, rhs)| Expr::BinOp {
                lhs: Box::new(lhs),
                op: BinOp::try_from(op).unwrap(),
                rhs: Box::new(rhs),
            });

        let sum = product
            .clone()
            .then(one_of("+-").then(product).repeated())
            .foldl(|lhs, (op, rhs)| Expr::BinOp {
                lhs: Box::new(lhs),
                op: BinOp::try_from(op).unwrap(),
                rhs: Box::new(rhs),
            });

        sum
    });

    let var_decl = text::keyword("let")
        .ignore_then(ident)
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .map(|(name, rhs)| Expr::Let {
            name,
            rhs: Box::new(rhs),
        });

    let fn_decl = text::keyword("fn")
        .ignore_then(ident)
        .then(ident.repeated())
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .map(|((name, args), body)| Expr::Fn {
            name,
            args,
            body: Box::new(body),
        });

    let decl = var_decl.or(fn_decl).or(expr).padded();

    decl.repeated()
        .map(|exprs| Program { exprs })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser(test_loc: &str, source: &str, expected: &str) {
        let file_name = "test";

        match parser().parse(source) {
            Ok(actual) => {
                assert_eq!(actual.to_string(), expected);
            }
            Err(e) => {
                let mut out = Vec::new();
                for report in make_reports(file_name, e) {
                    report
                        .write((file_name, ariadne::Source::from(source)), &mut out)
                        .unwrap();
                }
                let out = String::from_utf8(out).unwrap();
                panic!(
                    "Expected successful parse on test {}:{}\n{out}",
                    file!(),
                    test_loc
                );
            }
        }
    }

    macro_rules! test_parser {
        ($source:expr,$expected:expr,) => {
            test_parser!($source, $expected)
        };
        ($source:expr,$expected:expr) => {
            test_parser(&format!("{}:{}", line!(), column!()), $source, $expected);
        };
    }

    #[test]
    fn t_parse_op() {
        test_parser!("--a", "--a\n");
        test_parser!("\n-\n\n   a \n", "-a\n");
        test_parser!("  -a  \n\n  ", "-a\n");
    }

    #[test]
    fn t_parse_product() {
        test_parser!("a * - bb", "(a * -bb)\n");
        test_parser!("a / bb * - ccc", "((a / bb) * -ccc)\n");
    }

    #[test]
    fn t_parse_sum() {
        test_parser!("a / bb + - ccc", "((a / bb) + -ccc)\n");
    }

    #[test]
    fn t_parse_let() {
        test_parser!("let a = a; a + a", "let a = a;\n(a + a)\n");
    }

    #[test]
    fn t_parse_big() {
        test_parser!(
            r#"
                let a = a;
                let b = a + b;

                fn add x y = x + y;

                add(a * b, b)

            "#,
            &vec![
                "let a = a;",
                "let b = (a + b);",
                "fn add x y = (x + y);",
                "(add)((a * b), b)",
                "",
            ]
            .join("\n"),
        );
    }
}
