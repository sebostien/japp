use std::ops::Range;

use ariadne::{Label, Report, ReportKind};
use japp_util::Spanned;

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

impl ParseError<'_> {
    pub fn make_report<'f>(&self, file_name: &'f str) -> Report<'f, (&'f str, Range<usize>)> {
        let mut report = Report::build(ReportKind::Error, (file_name, self.span.clone()));

        match &self.error {
            ErrorKind::Multi(_) => todo!(),
            ErrorKind::Nom(kind) => {
                report = report.with_label(
                    Label::new((file_name, self.span.clone())).with_message(format!("{kind:?}")),
                );
            }
            ErrorKind::InvalidPrecedence(_) => todo!(),
            ErrorKind::UnexpectedToken { found, expected } => {
                report = report.with_label(
                    Label::new((file_name, self.span.clone()))
                        .with_message(format!("Unexpected token '{found}', expected '{expected}'")),
                );
            }
            ErrorKind::DuplicateFixity { other } => {
                report = report.with_label(
                    Label::new((file_name, other.clone()))
                        .with_message("Fixity first defined here"),
                );
                report = report.with_label(
                    Label::new((file_name, self.span.clone()))
                        .with_message("Fixity later defined here"),
                );
            }
            ErrorKind::ExprParser { error: kind } => {
                report = report.with_label(
                    Label::new((file_name, self.span.clone())).with_message(format!("{kind:?}")),
                );
            }
            ErrorKind::Mismatched {
                start: _,
                expected,
                extra_info,
            } => {
                report = report.with_label(
                    Label::new((file_name, self.span.clone())).with_message(extra_info),
                );

                if let Some(expected) = expected {
                    // TODO: Fix error
                    report = report.with_label(
                        Label::new((file_name, expected.span.clone())).with_message(format!(
                            "Expected here::: {:?} ::: {}",
                            expected.inner, extra_info
                        )),
                    );
                }
            }
        };

        report.finish()
    }
}
