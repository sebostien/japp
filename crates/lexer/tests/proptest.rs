use lexer::Lexer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn no_tokens(source in r".+\w.+") {
        println!("source = {source:?}");

        let lexer = Lexer::compile::<_, &&str>(&[]);
        let mut scanner = lexer.scan(0, &source);

        let mut prev_start = -1;
        let mut prev_end = -1;

        while let Some(token) = scanner.next() {
            // Tokens can't contain whitespace
            prop_assert!(token.inner.chars().all(|c| !c.is_whitespace()));

            // Tokens must be non empty
            prop_assert!(!token.span.is_empty());
            prop_assert!(!token.inner.is_empty());

            // The start of a token must be after the previous
            prop_assert!(token.span.start as isize >= prev_start);
            prev_start = token.span.start as isize;

            // The end of a token must be after the previous
            prop_assert!(token.span.end as isize >= prev_end);
            prev_end = token.span.end as isize;

            // The span in the source should match the token
            prop_assert_eq!(&source[token.span], token.inner);
        }

        // The final token should be at the end of the input (ignoring ending whitespace).
        let source = source.trim_end();
        prop_assert_eq!(source.len() as isize, prev_end);
    }
}

proptest! {

    // TODO: Seems to be too slow for large input
    #![proptest_config(
        ProptestConfig {
            // timeout: 1_000,
            cases: 1,
            ..Default::default()
        }
    )]
    #[test]
    fn arbitrary_token(token in r"(\w|\d){1,2}", source in r".\w.{1,3}") {
        println!("token = {token}, source = {source:?}");

        let lexer = Lexer::compile(&[token.as_str()]);
        let mut scanner = lexer.scan(0, &source);

        let mut prev_start = -1;
        let mut found_tokens = 0;
        let mut prev_end = -1;

        while let Some(matched) = scanner.next() {
            if matched.inner == token {
                found_tokens += 1;
            }

            // Tokens can't contain whitespace
            prop_assert!(matched.inner.chars().all(|c| !c.is_whitespace()));

            // Tokens must be non empty
            prop_assert!(!matched.span.is_empty());
            prop_assert!(!matched.inner.is_empty());

            // The start of a token must be after the previous
            prop_assert!(matched.span.start as isize >= prev_start);
            prev_start = matched.span.start as isize;

            // The end of a token must be after the previous
            prop_assert!(matched.span.end as isize >= prev_end);
            prev_end = matched.span.end as isize;

            // The span in the source should match the token
            prop_assert_eq!(&source[matched.span], matched.inner);
        }

        let mut actual = 0;
        let mut start = 0;
        while let Some(s) = source[start..].find(&token) {
            actual += 1;
            start = s;
        }

        prop_assert_eq!(actual, found_tokens);

        // The final token should be at the end of the input (ignoring ending whitespace).
        let source = source.trim_end();
        prop_assert_eq!(source.len() as isize, prev_end, "Len: {}, Last span end: {}", source.len(), prev_end);
    }
}
