use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lexer::Lexer;

fn build() -> Lexer {
    Lexer::compile(black_box(&[
        "struct",
        "let",
        "fn",
        "type",
        "enum",
        "letfn",
        "fntype",
        "structure",
    ]))
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Build lexer", |b| b.iter(|| build()));

    let lexer = build();
    let input = "".split("struct let fn type enum letfn fntype structure structurize fnlet typingenum hello world").collect::<Vec<_>>();

    c.bench_function("Run lexer", |b| {
        b.iter(|| {
            for input in black_box(&input) {
                if let Some(a) = lexer.find_match(input) {
                    assert!(!a.is_empty());
                }
            }
        })
    });
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
