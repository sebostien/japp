use japp_util::Spanned;
use lexer::Lexer;
use rand::Rng;
use std::ops::Range;

fn test_lexer(tokens: &[&str], source: &str, expected: &[(Range<usize>, &str)]) {
    let offset = rand::thread_rng().gen_range(0..100_000);

    println!("Lexing:\n\t'{source}'\nWith tokens:\n\t{tokens:?}");

    let lexer = Lexer::compile(tokens.into_iter().map(|s| *s));
    let mut scanner = lexer.scan(offset, source);

    for (span, inner) in expected.into_iter().cloned() {
        let token = scanner.next();
        assert_eq!(
            token,
            Some(Spanned {
                span: offset + span.start..offset + span.end,
                inner
            })
        );
        let token = token.unwrap();
        assert_eq!(token.span.len(), token.inner.len());
    }

    let last = scanner.next();
    assert!(last.is_none(), "Expected `None` but found '{:?}'", last);
}

#[test]
fn lexer() {
    let nfa = Lexer::compile(&["let", "fn", "lefn", "letfn"]);

    assert_eq!(None, nfa.find_match("l"));
    assert_eq!(Some("let"), nfa.find_match("let"));
    assert_eq!(Some("fn"), nfa.find_match("fn"));
    assert_eq!(Some("letfn"), nfa.find_match("letfn"));
    assert_eq!(Some("lefn"), nfa.find_match("lefn"));
    assert_eq!(Some("fn"), nfa.find_match("fnlet"));
    assert_eq!(Some("letfn"), nfa.find_match("letfnn"));
    assert_eq!(Some("let"), nfa.find_match("letffn"));
}

#[test]
fn empty() {
    test_lexer(&[], "", &[]);
    test_lexer(&["a"], "", &[]);
    test_lexer(&[], "testing", &[(0..7, "testing")]);
    test_lexer(
        &[],
        " \n \t a \n abc\t hello   ",
        &[(5..6, "a"), (9..12, "abc"), (14..19, "hello")],
    );
}

#[test]
fn longer() {
    let symbols = &[
        "struct",
        "fntype",
        "fn",
        "type",
        "enum",
        "letfn",
        "let",
        "structure",
    ];
    let lexer = Lexer::compile(symbols);
    let input = "".split("struct let fn type enum letfn fntype structure structurize fnlet typingenum hello world").collect::<Vec<_>>();

    for input in input {
        if let Some(a) = lexer.find_match(input) {
            assert!(symbols.contains(&a));
        } else {
            assert!(!symbols.contains(&input));
        }
    }
}

#[test]
fn simple_tokens() {
    test_lexer(
        &["a", "b", "ab"],
        "a ab aa aabb  \n b",
        &[
            (0..1, "a"),
            (2..4, "ab"),
            (5..6, "a"),
            (6..7, "a"),
            (8..9, "a"),
            (9..11, "ab"),
            (11..12, "b"),
            (16..17, "b"),
        ],
    );
}

#[test]
fn longer_tokens() {
    test_lexer(
        &["aa", "bb", "aabbcc", "c"],
        "\n\n\t   abc abb cc babaabbccaabbccc   \n",
        &[
            (6..8, "ab"),
            (8..9, "c"),
            (10..11, "a"),
            (11..13, "bb"),
            (14..15, "c"),
            (15..16, "c"),
            (17..20, "bab"),
            (20..26, "aabbcc"),
            (26..32, "aabbcc"),
            (32..33, "c"),
        ],
    );
}

#[test]
fn unicode() {
    test_lexer(
        &["Ѩ"],
        "ѨጒAጒ",
        &[
            (0.."Ѩ".len(), "Ѩ"),
            ("Ѩ".len().."Ѩ".len() + "ጒAጒ".len(), "ጒAጒ"),
        ],
    );
    test_lexer(
        &["Ѩ"],
        "𑤉Ѩ",
        &[(0.."𑤉".len(), "𑤉"), ("𑤉".len().."𑤉".len() + "Ѩ".len(), "Ѩ")],
    );
    test_lexer(
        &["w"],
        "wC:\u{1c645}w",
        &[
            (0..1, "w"),
            (1..1 + "C:\u{1c645}".len(), "C:\u{1c645}"),
            (1 + "C:\u{1c645}".len()..2 + "C:\u{1c645}".len(), "w"),
        ],
    );
    test_lexer(
        &["Ⱥ"],
        " \u{abc}Ⱥ",
        &[
            (1..1 + "\u{abc}".len(), "\u{abc}"),
            (1 + "\u{abc}".len()..1 + "\u{abc}".len() + "Ⱥ".len(), "Ⱥ"),
        ],
    );
}
