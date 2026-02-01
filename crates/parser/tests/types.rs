use parser::parse;

fn test_program(source: &str, expected: &str) {
    let ast = parse(source);

    match ast {
        Ok(ast) => assert_eq!(
            ast.to_string(),
            expected,
            "Expected:\n{ast}\nTo equal:\n{expected}"
        ),
        Err(err) => {
            let mut error_text = Vec::new();
            for e in err {
                e.make_report("test_file")
                    .write(
                        ("test_file", ariadne::Source::from(source)),
                        &mut error_text,
                    )
                    .unwrap();
            }
            panic!(
                "Expected Ok parse but found errors:\n{}",
                String::from_utf8(error_text).unwrap()
            );
        }
    }
}

#[test]
fn fn_factorial() {
    test_program(
        r#"
            infixr * 2;
            infixr - 1;
            fac : i32 -> i32 -> i32 ;
            fn fac n = match n {
                0 -> 1 ;
                n -> n * fac (n - 1) ;
            } ;
        "#,
        &vec![
            // "fac : i32 -> i32 -> i32 ;",
            "fn fac n = match n { 0 -> 1 ; n -> ( n * fac ( ( n - 1 ) ) ) ; } ;",
        ]
        .join("\n"),
    );
}

#[test]
fn fn_nested_ty() {
    test_program(
        r#"
            infixr :: 2;
            map : ( X -> Y ) -> List<X> -> List<Y> ;
            fn map f xs = match xs {
                [] -> [];
                xs -> f(head(xs)) :: map(f, tail(xs) ) ;
            } ;
        "#,
        &vec![
            // "map : ( X -> Y ) -> List<X> -> List<Y> ;",
            "fn map f xs = match xs { [] -> [] ; xs -> ( f ( head ( xs ) ) :: map ( f , tail ( xs ) ) ) ; } ;",
        ]
        .join("\n"),
    );
}
