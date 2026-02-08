use macro_japp::test_glob;
use spressions::{Spression, ToSpression};
use std::fs;
use std::path::Path;

const TEST_DIVIDER: &'static str = "=====";
const SECTION_DIVIDER: &'static str = "-----";

#[derive(Debug)]
struct ParseTest {
    test_name: String,
    source: String,
    expected: Spression,
}

fn trim(s: &str) -> &str {
    s.trim_matches(|c: char| c == '-' || c == '=' || c.is_whitespace())
}

// TODO: Add tags to tests (should_fail, etc...)
fn parse_file(path: &Path) -> Vec<ParseTest> {
    let contents = fs::read_to_string(path).expect("Could not read file");
    let tests = contents.trim().split(TEST_DIVIDER).collect::<Vec<_>>();

    tests
        .into_iter()
        .filter(|v| !v.is_empty())
        .map(|t| {
            let sections = t
                .split(SECTION_DIVIDER)
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>();

            match sections[..] {
                [test_name, source, expected] => ParseTest {
                    test_name: trim(test_name).to_string(),
                    source: trim(source).to_string(),
                    expected: trim(expected).parse().expect("Spression is not valid"),
                },
                _ => panic!(
                    "There should be exectly 3 sections in a test. Found {}",
                    sections.len()
                ),
            }
        })
        .collect()
}

#[test_glob("corpus/*.in")]
fn test_corpus(p: &Path) {
    let mut failed = false;

    for ParseTest {
        test_name,
        source,
        expected,
    } in parse_file(p)
    {
        let parsed = parser::parse(&source).unwrap().to_spression();

        if parsed != expected {
            eprintln!(
                "{}",
                vec![
                    &format!("Wrong parse found in: {test_name}"),
                    "Expected:",
                    "===",
                    &parsed.to_string(),
                    "===",
                    "to equal:",
                    "===",
                    &expected.to_string(),
                    "===",
                ]
                .join("\n")
            );
            failed = true;
        }
    }

    if failed {
        panic!("Some tests failed, see output above!");
    }
}
