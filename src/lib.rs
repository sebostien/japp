use ariadne::{Label, Report, ReportKind};
use std::ops::Range;

mod ast;
mod expr_parser;
mod lexer;
mod parser;

pub use parser::parse;
use parser::{ErrorKind, ParseError};

pub fn nom_make_reports<'f>(
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
            };

            report.finish()
        })
        .collect()
}

// pub fn make_reports<'f>(
//     file_name: &'f str,
//     errors: &[Simple<char>],
// ) -> Vec<Report<'f, (&'f str, Range<usize>)>> {
//     errors
//         .iter()
//         .map(|error| {
//             let mut report = Report::build(ReportKind::Error, (file_name, error.span()));
//
//             let message = match error.reason() {
//                 SimpleReason::Unexpected => {
//                     report = report.with_message("Unexpected token");
//                     error.to_string()
//                 }
//                 SimpleReason::Unclosed { span, delimiter } => todo!(
//                     "Check if the following is a good message: {:?} {} ::: {:?} ::: {}",
//                     error,
//                     error,
//                     span,
//                     delimiter
//                 ),
//                 SimpleReason::Custom(e) => e.clone(),
//             };
//
//             report
//                 .with_label(Label::new((file_name, error.span())).with_message(message))
//                 .finish()
//         })
//         .collect()
// }
