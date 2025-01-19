use japp_util::Spanned;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{digit1, multispace1, space1};
use nom::combinator::eof;
use nom::multi::many0;
use nom::Parser;
use std::ops::Range;

use crate::ast::{Assoc, Fixity, UnparsedDecl, UnparsedProgram};

#[derive(Debug)]
pub struct ParseError<'source> {
    pub span: Range<usize>,
    pub error: ErrorKind<'source>,
}

#[derive(Debug)]
pub enum ErrorKind<'source> {
    Multi(Vec<Self>),
    Nom(nom::error::ErrorKind),
    InvalidPrecedence(String),
    UnexpectedToken {
        found: &'source str,
        expected: &'source str,
    },
    DuplicateFixity {
        other: Range<usize>,
    },
    ExprParser {
        error: String,
    },
    Mismatched {
        start: &'source str,
        expected: Option<Spanned<&'source str>>,
        extra_info: &'source str,
    },
}

impl<'a> nom::error::ParseError<Source<'a>> for ParseError<'_> {
    fn from_error_kind(input: Source<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            span: input.byte_offset()..input.byte_offset() + 1,
            error: ErrorKind::Nom(kind),
        }
    }

    fn append(input: Source<'a>, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.span.start = other.span.start.min(input.byte_offset());
        other.span.end = other.span.end.max(input.byte_offset());

        if let ErrorKind::Multi(errors) = &mut other.error {
            errors.push(ErrorKind::Nom(kind));
        } else {
            other.error = ErrorKind::Multi(vec![ErrorKind::Nom(kind), other.error]);
        }
        other
    }
}

type Source<'a> = nom_span::Spanned<&'a str>;
type IResult<'a, O> = nom::IResult<Source<'a>, O, ParseError<'a>>;

pub fn convert_nom_error<O>(value: nom::IResult<Source, O>) -> IResult<O>
where
    O: std::fmt::Debug,
{
    if value.is_err() {
        println!("convert_nom_error:: {value:?}");
    }

    match value {
        Ok(v) => Ok(v),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(ParseError {
            span: e.input.byte_offset()..e.input.byte_offset(),
            error: ErrorKind::Nom(e.code),
        })),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Error(ParseError {
            span: e.input.byte_offset()..e.input.byte_offset(),
            error: ErrorKind::Nom(e.code),
        })),
        Err(nom::Err::Incomplete(_)) => panic!("Got Incomplete error"),
    }
}

fn spaces(input: Source) -> Source {
    take_till::<_, _, ()>(|c: char| !c.is_whitespace())(input)
        .map(|(input, _)| input)
        .unwrap_or(input)
}

/// Any utf-8 (without whitespace) sequence not starting with an ascii digit.
fn ident(input: Source) -> IResult<Source> {
    let input = spaces(input);

    let ident_start = input.byte_offset();

    if input.starts_with(",") {
        return Err(nom::Err::Error(ParseError {
            span: input.byte_offset()..input.byte_offset() + 1,
            error: ErrorKind::UnexpectedToken {
                found: ",",
                expected: "Ident",
            },
        }));
    }

    let (input, ident) = convert_nom_error(take_till(|c: char| c.is_whitespace())(input))?;

    let ident_end = input.byte_offset();

    let (input, _) = convert_nom_error(many0(space1)(input))?;

    match *ident.data() {
        ";" => Err(nom::Err::Failure(ParseError {
            span: ident_start..ident_end + 1,
            error: ErrorKind::UnexpectedToken {
                found: ident.data(),
                expected: "Ident",
            },
        })),
        "=" => Err(nom::Err::Error(ParseError {
            span: ident_start..ident_end + 1,
            error: ErrorKind::UnexpectedToken {
                found: ident.data(),
                expected: "Ident",
            },
        })),
        _ => Ok((input, ident)),
    }
}

fn unparsed_expr(input: Source) -> IResult<Source> {
    let input = spaces(input);
    let (input, expr) = convert_nom_error(take_till(|c| c == ';')(input))?;
    Ok((input, expr))
}

fn infix_decl(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, assoc) = alt((
        tag("infixl").map(|_| Assoc::Left),
        tag("infixr").map(|_| Assoc::Right),
        tag("infix").map(|_| Assoc::None),
    ))(input)?;
    let input = spaces(input);

    let (input, ident) = ident(input)?;
    let (input, prec) = digit1(input)?;
    let input = spaces(input);

    let prec = prec.parse::<usize>().map_err(|_| {
        nom::Err::Failure(ParseError {
            span: prec.byte_offset()..prec.byte_offset() + prec.len(),
            error: ErrorKind::InvalidPrecedence(prec.to_string()),
        })
    })?;

    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        UnparsedDecl::Infix {
            ident,
            fixity: Fixity { prec, assoc },
        },
    ))
}

fn let_decl(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, _) = tag("let")(input)?;
    let input = spaces(input);

    let (input, ident) = ident(input)?;
    let input = spaces(input);
    let (input, _) = tag("=")(input)?;
    let input = spaces(input);

    let (input, expr) = unparsed_expr(input)?;
    let input = spaces(input);
    let (input, _) = tag(";")(input)?;

    Ok((input, UnparsedDecl::Let { ident, rhs: expr }))
}

fn fn_decl(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, _) = tag("fn")(input)?;
    let (input, _) = space1(input)?;

    let (input, name) = ident(input)?;
    let (input, args) = many0(ident)(input)?;
    let (input, _) = tag("=")(input)?;

    let (input, body) = unparsed_expr(input)?;
    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        UnparsedDecl::Fn {
            ident: name,
            args,
            body,
        },
    ))
}

fn decl(input: Source) -> IResult<UnparsedDecl> {
    alt((infix_decl, let_decl, fn_decl))(input)
}

pub fn parse_program(input: &str) -> IResult<UnparsedProgram> {
    let input = Source::new_for_ut8(input);

    let (input, decls) = many0(decl)(input)?;
    let (input, _) = many0(multispace1)(input)?;
    let (input, _) = eof(input)?;

    Ok((input, UnparsedProgram { decls }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn t_let() {
        assert!(let_decl(Source::new_for_ut8("let a = -   -a;")).is_ok());
        let a = let_decl(Source::new_for_ut8("\nlet \n c = \n -\n\n   a \n;"));
        assert!(a.is_ok(), "{a:?}");
        assert!(let_decl(Source::new_for_ut8("let z =  -a  \n\n; ")).is_ok());
        assert!(let_decl(Source::new_for_ut8("let a23 = a + b ;")).is_ok());
        assert!(let_decl(Source::new_for_ut8("let b = a + b ;")).is_ok());
        assert!(let_decl(Source::new_for_ut8(
            "let add = 1+(2 + (3+((4)))) == (1 + 2 + 3 +4);"
        ))
        .is_ok());
        assert!(let_decl(Source::new_for_ut8("let abc = a + b * c /(2/d) ;")).is_ok());
    }

    #[test]
    fn t_fn() {
        assert!(fn_decl(Source::new_for_ut8("\n fn add x y = x + y; ")).is_ok());
    }

    #[test]
    fn t_partial_application() {
        assert!(fn_decl(Source::new_for_ut8(
            "fn add x y = x + y; let z = add(x)(y);"
        ))
        .is_ok());
    }

    #[test]
    fn t_infix() {
        assert!(infix_decl(Source::new_for_ut8(" infix asdkj23lka9* 10;")).is_ok());
        assert!(infix_decl(Source::new_for_ut8("\ninfixl jasdk 10;")).is_ok());
        assert!(infix_decl(Source::new_for_ut8("infixr asld 10;")).is_ok());
        assert!(infix_decl(Source::new_for_ut8("infixr >=> 0;")).is_ok());
    }

    #[test]
    fn t_idents() {
        // Reserved
        assert!(ident(Source::new_for_ut8(" = ")).is_err());
        assert!(ident(Source::new_for_ut8(" ; ")).is_err());

        // Ok
        assert!(ident(Source::new_for_ut8("hello")).is_ok());
        assert!(ident(Source::new_for_ut8("=;=")).is_ok());

        let (input, id) = ident(Source::new("fn . ", true)).unwrap();

        assert_eq!(*input.data(), ". ");
        assert_eq!(input.line(), 1);
        assert_eq!(input.col(), 4);
        assert_eq!(input.byte_offset(), 3);

        assert_eq!(*id.data(), "fn");
        assert_eq!(id.col(), 1);
        assert_eq!(id.line(), 1);
        assert_eq!(id.byte_offset(), 0);
    }

    // //////////////////////////////////////////////////
    // ///// Errors ////////////////////////////////////
    // ////////////////////////////////////////////////

    fn test_parser_err(test_loc: &str, source: &str, reason: &str) {
        match parse(source) {
            Ok(actual) => {
                panic!(
                    "Parser successfully parsed test {}:{test_loc}\nWith result:\n'''\n{actual}\n'''\nBut this should fail because: {reason}",
                    file!()
                );
            }
            Err(_) => {
                // TODO: Validate error message
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

    #[test]
    fn t_error_multi_fixity() {
        test_parser_err!(
            "infixr a 0; infix a 1;",
            "Precedence too big. Must fit in usize"
        );
    }
}
