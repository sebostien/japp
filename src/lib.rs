use ariadne::{Label, Report, ReportKind};
use parser::{ErrorKind, ParseError};
use std::ops::Range;

#[derive(Debug)]
pub enum ErrorCode {
    CommandError = 4,
    LogSetup = 5,
    ParseError = 30,
    IoError = 40,
    NoInputFile = 41,
    FileNotFound = 42,
}

pub fn make_parse_reports<'f>(
    file_name: &'f str,
    errors: &[ParseError],
) -> Vec<Report<'f, (&'f str, Range<usize>)>> {
    errors
        .iter()
        .map(|error| {
            let mut report = Report::build(ReportKind::Error, (file_name, error.span.clone()));

            match &error.error {
                ErrorKind::Multi(_) => todo!(),
                ErrorKind::Nom(kind) => {
                    report = report.with_label(
                        Label::new((file_name, error.span.clone()))
                            .with_message(format!("{kind:?}")),
                    );
                }
                ErrorKind::InvalidPrecedence(_) => todo!(),
                ErrorKind::UnexpectedToken { found, expected } => {
                    report = report.with_label(
                        Label::new((file_name, error.span.clone())).with_message(format!(
                            "Unexpected token '{found}', expected '{expected}'"
                        )),
                    );
                }
                ErrorKind::DuplicateFixity { other } => {
                    report = report.with_label(
                        Label::new((file_name, other.clone()))
                            .with_message("Fixity first defined here"),
                    );
                    report = report.with_label(
                        Label::new((file_name, error.span.clone()))
                            .with_message("Fixity later defined here"),
                    );
                }
                ErrorKind::ExprParser { error: kind } => {
                    report = report.with_label(
                        Label::new((file_name, error.span.clone()))
                            .with_message(format!("{kind:?}")),
                    );
                }
                ErrorKind::Mismatched {
                    start,
                    expected,
                    extra_info,
                } => {
                    report = report.with_label(
                        Label::new((file_name, error.span.clone())).with_message(extra_info),
                    );

                    if let Some(expected) = expected {
                        // TODO: Fix error. Showing wrong location
                        report = report.with_label(
                            Label::new((file_name, start.span.clone())).with_message(format!(
                                "Expected here::: {start:?} ::: token = {expected} ::: {extra_info}",
                            )),
                        );
                    }
                }
                ErrorKind::UnexpectedEof { expected } => {
                    report = report.with_label(
                        Label::new((file_name, error.span.clone()))
                            .with_message(format!("Unexpected EOF, expected '{expected}'")),
                    );
                }
            };

            report.finish()
        })
        .collect()
}
