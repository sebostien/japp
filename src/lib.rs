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

use expr::{BinOp, Expr};
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

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
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

    let decl = recursive(|decl| {
        let var_decl = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map(|((name, rhs), then)| Expr::Let {
                name,
                rhs: Box::new(rhs),
                then: Box::new(then),
            });

        let fn_decl = text::keyword("fn")
            .ignore_then(ident)
            .then(ident.repeated())
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl)
            .map(|(((name, args), body), then)| Expr::Fn {
                name,
                args,
                body: Box::new(body),
                then: Box::new(then),
            });

        var_decl.or(fn_decl).or(expr).padded()
    });

    decl.then_ignore(end())
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
                panic!("Expected successful parse on test {}:{}\n{out}", file!(), test_loc);
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
        test_parser!("--a", "--a");
        test_parser!("\n-\n\n   a \n", "-a");
        test_parser!("  -a  \n\n  ", "-a");
    }

    #[test]
    fn t_parse_product() {
        test_parser!("a * - bb", "(a * -bb)");
        test_parser!("a / bb * - ccc", "((a / bb) * -ccc)");
    }

    #[test]
    fn t_parse_sum() {
        test_parser!("a / bb + - ccc", "((a / bb) + -ccc)");
    }

    #[test]
    fn t_parse_let() {
        test_parser!("let a = a; a + a", "let a = a;\n(a + a)");
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
