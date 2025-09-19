use std::path::PathBuf;

use clap::Parser;
use japp::ErrorCode;
use parser::parse;

#[derive(Debug, Clone, clap::Parser)]
enum CLI {
    Compile {
        file_name: PathBuf,
        #[clap(short, long)]
        out_name: Option<PathBuf>,
    },
}

fn main() -> Result<(), ErrorCode> {
    let command = CLI::try_parse().map_err(|e| {
        eprintln!("{e}");
        ErrorCode::CommandError
    })?;

    match command {
        CLI::Compile {
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
                eprintln!("{e}");
                ErrorCode::FileNotFound
            })?;
            let source = source.as_str();

            match parse(source) {
                Ok(source) => {
                    // println!("{source:#?}\n");
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
