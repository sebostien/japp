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

mod decl;
mod expr;
mod program;

use decl::Decl;
use expr::{BinOp, Expr};
use program::Program;
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

fn ident() -> impl Parser<char, String, Error = Simple<char>> {
    text::ident().padded()
}

fn call<'a>(
    expr: Recursive<'a, char, Expr, Simple<char>>,
) -> impl Parser<char, Expr, Error = Simple<char>> + 'a {
    ident()
        .then(
            expr.clone()
                .separated_by(just(','))
                .allow_trailing()
                .delimited_by(just('('), just(')')),
        )
        .map(|(f, args)| Expr::Call(Box::new(Expr::Ident(f)), args))
}

fn atom<'a>(
    expr: Recursive<'a, char, Expr, Simple<char>>,
) -> impl Parser<char, Expr, Error = Simple<char>> + 'a {
    expr.clone()
        .delimited_by(just('('), just(')'))
        .or(call(expr))
        .or(ident().map(Expr::Ident))
}

fn unary<'a>(
    expr: Recursive<'a, char, Expr, Simple<char>>,
) -> impl Parser<char, Expr, Error = Simple<char>> + 'a {
    let op = |c| just(c).padded();

    op('-')
        .clone()
        .repeated()
        .then(atom(expr.clone()))
        .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)))
}

fn product<'a>(
    expr: Recursive<'a, char, Expr, Simple<char>>,
) -> impl Parser<char, Expr, Error = Simple<char>> + 'a {
    unary(expr.clone())
        .then(one_of("*/").then(unary(expr.clone())).repeated())
        .foldl(|lhs, (op, rhs)| Expr::BinOp {
            lhs: Box::new(lhs),
            op: BinOp::try_from(op).unwrap(),
            rhs: Box::new(rhs),
        })
}

fn sum<'a>(
    expr: Recursive<'a, char, Expr, Simple<char>>,
) -> impl Parser<char, Expr, Error = Simple<char>> + 'a {
    product(expr.clone())
        .then(one_of("+-").then(product(expr.clone())).repeated())
        .foldl(|lhs, (op, rhs)| Expr::BinOp {
            lhs: Box::new(lhs),
            op: BinOp::try_from(op).unwrap(),
            rhs: Box::new(rhs),
        })
}

fn expr() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| sum(expr))
}

fn decl() -> impl Parser<char, Decl, Error = Simple<char>> {
    let var_decl = text::keyword("let")
        .ignore_then(ident())
        .then_ignore(just('='))
        .then(expr())
        .then_ignore(just(';'))
        .map(|(ident, rhs)| Decl::Let { ident, rhs });

    let fn_decl = text::keyword("fn")
        .ignore_then(ident())
        .then(ident().repeated())
        .then_ignore(just('='))
        .then(expr())
        .then_ignore(just(';'))
        .map(|((ident, args), body)| Decl::Fn { ident, args, body });

    var_decl
        .or(fn_decl)
        .or(expr().then_ignore(just(";")).map(Decl::Expr))
        .padded()
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    decl()
        .repeated()
        .then(expr().padded().or_not())
        .map(|(exprs, last)| Program { exprs, last })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser(test_loc: &str, source: &str, expected: &str) {
        let file_name = "test";

        match parser().parse(source) {
            Ok(actual) => {
                assert_eq!(
                    actual.to_string(),
                    expected,
                    "Not the expected result from test {}:{test_loc}",
                    file!()
                );
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
                    "Expected successful parse on test {}:{test_loc}\n{out}",
                    file!()
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
        test_parser!(
            "a / bb + - ccc * a + b - -(b * c);",
            "((((a / bb) + (-ccc * a)) + b) - -(b * c));"
        );
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
            ]
            .join("\n"),
        );
    }
}
