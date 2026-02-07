use std::ops::Range;

use spressions::{Spression, ToSpression};

enum Expr {
    Add {
        span: Range<usize>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Mul {
        span: Range<usize>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Lit(Lit),
}

enum Lit {
    Int { span: Range<usize>, num: isize },
}

impl ToSpression for Expr {
    fn to_spression(self) -> Spression {
        match self {
            Self::Add { span, lhs, rhs } => Spression {
                node: "Add".to_string(),
                span: Some(span),
                data: Vec::new(),
                children: vec![lhs.to_spression(), rhs.to_spression()],
            },
            Self::Mul { span, lhs, rhs } => Spression {
                node: "Mul".to_string(),
                span: Some(span),
                data: Vec::new(),
                children: vec![lhs.to_spression(), rhs.to_spression()],
            },
            Self::Lit(lit) => Spression {
                node: "Lit".to_string(),
                span: None,
                data: Vec::new(),
                children: vec![lit.to_spression()],
            },
        }
    }
}

impl ToSpression for Lit {
    fn to_spression(self) -> Spression {
        match self {
            Self::Int { span, num } => Spression {
                node: "Int".to_string(),
                span: Some(span),
                data: vec![num.to_string()],
                children: vec![],
            },
        }
    }
}

#[test]
fn simple_ast_spression() {
    let source = r#"
        (Add (Lit (Int 1):0..1 ) (Lit (Int 2):9..10 ) ):0..10
    "#;

    println!("{}", &source[40..]);

    let expected = Expr::Add {
        span: 0..10,
        lhs: Box::new(Expr::Lit(Lit::Int { span: 0..1, num: 1 })),
        rhs: Box::new(Expr::Lit(Lit::Int {
            span: 9..10,
            num: 2,
        })),
    };

    let source = source.parse();
    let expected = expected.to_spression();

    if source.as_ref() != Ok(&expected) {
        panic!(
            "Expected:\n{:#?}\nto equal:\n{expected:#?}",
            source.unwrap()
        );
    }
}

#[test]
fn ast_spression() {
    let source = r#"
        (Add
            (Mul (Lit (Int 1): 0 ..1  ) (Lit (Int 2): 9 ..10 ) ): 0..10
            (Add (Lit (Int 3): 40..41 ) (Lit (Int 4): 54..55 ) ):40..55
        ):0..55
    "#;

    println!("{}", &source[40..]);

    let expected = Expr::Add {
        span: 0..55,
        lhs: Box::new(Expr::Mul {
            span: 0..10,
            lhs: Box::new(Expr::Lit(Lit::Int { span: 0..1, num: 1 })),
            rhs: Box::new(Expr::Lit(Lit::Int {
                span: 9..10,
                num: 2,
            })),
        }),
        rhs: Box::new(Expr::Add {
            span: 40..55,
            lhs: Box::new(Expr::Lit(Lit::Int {
                span: 40..41,
                num: 3,
            })),
            rhs: Box::new(Expr::Lit(Lit::Int {
                span: 54..55,
                num: 4,
            })),
        }),
    };

    let source = source.parse();
    let expected = expected.to_spression();

    if source.as_ref() != Ok(&expected) {
        panic!(
            "Expected:\n{:#?}\nto equal:\n{expected:#?}",
            source.unwrap()
        );
    }
}
