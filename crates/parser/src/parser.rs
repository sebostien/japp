use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{digit1, multispace1, space1};
use nom::combinator::eof;
use nom::multi::{many0, separated_list1};
use nom::sequence::delimited;
use nom::{InputTakeAtPosition, Parser};
use std::cell::RefCell;

use crate::ast::{Associativity, Type, UnparsedProgram};
use crate::{ErrorKind, Fixity, Ident, Lit, ParseError, UnparsedDecl};
use japp_util::Spanned;

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
        Err(nom::Err::Incomplete(_)) => unreachable!("Got Incomplete error"),
    }
}

fn spaces(input: Source) -> Source {
    take_till::<_, _, ()>(|c: char| !c.is_whitespace())(input)
        .map(|(input, _)| input)
        .unwrap_or(input)
}

/// Any utf-8 (without whitespace) sequence not starting with an ascii digit.
fn ident(input: Source) -> IResult<Ident> {
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

    let (input, full_ident) = convert_nom_error(take_till(|c: char| c.is_whitespace())(input))?;
    let ident_span = full_ident.byte_offset()..full_ident.byte_offset() + full_ident.len();
    let ident = Ident::new(ident_span, full_ident.data());

    let ident_end = input.byte_offset();

    // Make sure a space is after the ident
    let (input, _) = convert_nom_error(many0(space1)(input))?;

    // Non valid idents
    if matches!(*full_ident, "=" | ";" | "," | "fn") {
        Err(nom::Err::Error(ParseError {
            span: ident_start..ident_end + 1,
            error: ErrorKind::UnexpectedToken {
                found: ident.outer(),
                expected: "Ident",
            },
        }))
    } else {
        Ok((input, ident))
    }
}

fn lit(input: Source) -> IResult<Spanned<Lit>> {
    match ident(input) {
        Ok((input, ident)) => Ok((
            input,
            Spanned {
                span: ident.outer_span(),
                inner: Lit::from(ident),
            },
        )),
        Err(e) => {
            // TODO: Attach some context
            Err(e)
        }
    }
}

fn unparsed_expr(input: Source) -> IResult<Source> {
    let input = spaces(input);

    // THIS IS SO FUCKING STUPID!
    let inc = RefCell::new(0usize);

    // Take as much as possible.
    // Only quitting for EOF or when ';' is found and we are not in a block.
    let (input, expr) = input.split_at_position_complete(move |c| {
        let x = *inc.borrow();
        match (x, c) {
            (0, ';') => {
                return true;
            }
            (x, '{') => {
                inc.replace(x + 1);
            }
            (x, '}') => {
                inc.replace(x.saturating_sub(1));
            }
            _ => {}
        }

        false
    })?;

    Ok((input, expr))
}

fn infix_decl(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, assoc) = alt((
        tag("infixl").map(|_| Associativity::Left),
        tag("infixr").map(|_| Associativity::Right),
        tag("infix").map(|_| Associativity::None),
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

fn const_decl(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, _) = tag("const")(input)?;
    let input = spaces(input);

    let (input, ident) = ident(input)?;
    let input = spaces(input);
    let (input, _) = tag("=")(input)?;
    let input = spaces(input);

    let (input, expr) = unparsed_expr(input)?;
    let input = spaces(input);
    let (input, _) = tag(";")(input)?;

    Ok((input, UnparsedDecl::Const { ident, rhs: expr }))
}

fn ty(input: Source) -> IResult<Spanned<Type>> {
    // TODO: Ident, Refined

    let input = spaces(input);
    let start = input.byte_offset();

    let inside = delimited(tag("("), tys, tag(")")).map(|inner| {
        let end = input.byte_offset();
        Spanned {
            span: start..end,
            inner: Type::Paren {
                inner: Box::new(inner),
            },
        }
    });

    let refined = ident
        .and(delimited(
            tag("<"),
            separated_list1(tag(","), tys),
            tag(">"),
        ))
        .map(|(ident, args)| {
            let end = input.byte_offset();
            Spanned::new(Type::Refined { ident, args }, start..end)
        });

    let single = ident.map(|id| {
        let end = input.byte_offset();
        Spanned::new(Type::Ident(id), start..end)
    });

    let (input, ty) = refined.or(inside).or(single).parse(input)?;
    let input = spaces(input);
    Ok((input, ty))
}

fn tys(input: Source) -> IResult<Spanned<Type>> {
    let input = spaces(input);
    let start = input.byte_offset();

    let many = separated_list1(tag("->"), ty).map(|args| {
        let end = input.byte_offset();
        Spanned {
            span: start..end,
            inner: Type::Fn { args },
        }
    });

    let (input, inner) = many.or(ty).parse(input)?;
    let input = spaces(input);

    Ok((input, inner))
}

/// `fac : isize -> isize`
fn fn_sig(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, name) = ident(input)?;

    let input = spaces(input);
    let (input, _) = tag(":")(input)?;
    let (input, _) = space1(input)?;

    let (input, sig) = tys(input)?;
    let input = spaces(input);
    let (input, _) = tag(";")(input)?;

    Ok((input, UnparsedDecl::FnSig { ident: name, sig }))
}

fn fn_decl(input: Source) -> IResult<UnparsedDecl> {
    let input = spaces(input);
    let (input, _) = tag("fn")(input)?;
    let (input, _) = space1(input)?;

    let (input, name) = ident(input)?;
    let args_start = input.byte_offset();
    let (input, args) = many0(ident)(input)?;
    let args_end = input.byte_offset();
    let (input, _) = tag("=")(input)?;

    let (input, body) = unparsed_expr(input)?;
    let input = spaces(input);
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
    alt((infix_decl, const_decl, fn_decl, fn_sig))(input)
}

pub fn parse_program(input: &str) -> IResult<'_, UnparsedProgram<'_>> {
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
    fn t_const() {
        assert!(const_decl(Source::new_for_ut8("const a = -   -a;")).is_ok());
        let a = const_decl(Source::new_for_ut8("\nconst \n c = \n -\n\n   a \n;"));
        assert!(a.is_ok(), "{a:?}");
        assert!(const_decl(Source::new_for_ut8("const z =  -a  \n\n; ")).is_ok());
        assert!(const_decl(Source::new_for_ut8("const a23 = a + b ;")).is_ok());
        assert!(const_decl(Source::new_for_ut8("const b = a + b ;")).is_ok());
        assert!(const_decl(Source::new_for_ut8(
            "const add = 1+(2 + (3+((4)))) == (1 + 2 + 3 +4);"
        ))
        .is_ok());
        assert!(const_decl(Source::new_for_ut8("const abc = a + b * c /(2/d) ;")).is_ok());
    }

    #[test]
    fn t_fn() {
        assert!(fn_decl(Source::new_for_ut8("\n fn add x y = x + y; ")).is_ok());
        assert!(fn_decl(Source::new_for_ut8(
            "fn add 0 y = y ;\n fn add x y = 1 + add (x - 1) y ; "
        ))
        .is_ok());
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
    fn t_match() {
        // Simple
        assert!(unparsed_expr(Source::new_for_ut8("match n { x -> x ; } ;")).is_ok());
        // Last semi in match should be optional
        assert!(unparsed_expr(Source::new_for_ut8("match n { x -> x };")).is_ok());
        // Multiple rows
        assert!(unparsed_expr(Source::new_for_ut8("match n { 1 -> 2 ; z -> z + 1 ; } ;")).is_ok());
    }

    #[test]
    fn t_idents() {
        // Reserved
        assert!(ident(Source::new_for_ut8(" = ")).is_err());
        assert!(ident(Source::new_for_ut8(" ; ")).is_err());

        // Ok
        assert!(ident(Source::new_for_ut8("hello")).is_ok());
        assert!(ident(Source::new_for_ut8("=;=")).is_ok());

        let (input, id) = ident(Source::new("_toho_ . ", true)).unwrap();

        assert_eq!(*input.data(), ". ");
        assert_eq!(input.line(), 1);
        assert_eq!(input.col(), 8);
        assert_eq!(input.byte_offset(), 7);

        assert_eq!(id.outer(), "_toho_");
        assert_eq!(id.outer_span(), 0..6);
        assert_eq!(id.inner(), "toho");
        assert_eq!(id.inner_span(), 1..5);
    }

    #[test]
    fn t_ok_type() {
        assert!(tys(Source::new_for_ut8("( X -> Y )")).is_ok());
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
        test_parser_err!("infixr a 0; infix a 1;", "Duplicate fixity declared.");
    }
}
