use clap::Parser;
use fern::log_file;
use japp::ErrorCode;
use log::{error, info};
use parser::parse;
use std::path::PathBuf;
use std::time::SystemTime;
use typecheck::typecheck;

#[derive(Debug, Clone, clap::Parser)]
enum Cli {
    Transpile {
        file_name: PathBuf,
        #[clap(short, long)]
        out_name: Option<PathBuf>,
    },
    Compile {
        file_name: PathBuf,
        #[clap(short, long)]
        out_name: Option<PathBuf>,
    },
}

fn main() -> Result<(), ErrorCode> {
    if let Err(e) = setup_logger() {
        eprintln!("{e}");
        return Err(ErrorCode::LogSetup);
    };

    let command = Cli::try_parse().map_err(|e| {
        error!("{e}");
        ErrorCode::CommandError
    })?;

    match command {
        Cli::Compile {
            file_name,
            out_name,
        } => {
            let file_name_str = file_name.to_string_lossy();
            let file_name_str = file_name_str.as_ref();

            let out_name = out_name.clone().unwrap_or_else(|| {
                let mut o = file_name.clone();
                o.set_extension("js");
                o
            });

            let source = std::fs::read_to_string(&file_name).map_err(|e| {
                error!("Could not read file!");
                error!("{e}");
                ErrorCode::FileNotFound
            })?;
            let source = source.as_str();

            match parse(source) {
                Ok(source) => {
                    println!("{:#?}", typecheck(source));
                    // println!("{}", compiler::compile(source).unwrap());
                    Ok(())
                }
                Err(e) => {
                    for report in japp::make_parse_reports(file_name_str, &e) {
                        if let Err(e) =
                            report.eprint((file_name_str, ariadne::Source::from(source)))
                        {
                            eprintln!("{e}");
                            return Err(ErrorCode::IoError);
                        }
                    }
                    Err(ErrorCode::ParseError)
                }
            }
        }
        Cli::Transpile {
            file_name,
            out_name,
        } => {
            let file_name_str = file_name.to_string_lossy();
            let file_name_str = file_name_str.as_ref();

            let out_name = out_name.clone().unwrap_or_else(|| {
                let mut o = file_name.clone();
                o.set_extension("js");
                o
            });

            let source = std::fs::read_to_string(&file_name).map_err(|e| {
                error!("Could not read file!");
                error!("{e}");
                ErrorCode::FileNotFound
            })?;
            let source = source.as_str();

            match parse(source) {
                Ok(source) => {
                    println!("{source:#?}\n");
                    info!("Writing file {}", out_name.to_string_lossy());
                    std::fs::write(out_name, transpiler::transpile(source)).unwrap();
                    Ok(())
                }
                Err(e) => {
                    for report in japp::make_parse_reports(file_name_str, &e) {
                        if let Err(e) =
                            report.eprint((file_name_str, ariadne::Source::from(source)))
                        {
                            eprintln!("{e}");
                            return Err(ErrorCode::IoError);
                        }
                    }
                    Err(ErrorCode::ParseError)
                }
            }
        }
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    let file_log = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(log_file("japp.log")?);

    let stderr_log = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stderr());

    fern::Dispatch::new()
        .chain(file_log)
        .chain(stderr_log)
        .apply()?;
    Ok(())
}
