use japp::parse;

#[derive(Debug)]
enum ErrorCode {
    ParseError = 30,
    IoError = 40,
    NoInputFile = 41,
    FileNotFound = 42,
}

fn main() -> Result<(), ErrorCode> {
    let file_name = std::env::args().nth(1).ok_or_else(|| {
        eprintln!("Pass input as argument!");
        ErrorCode::NoInputFile
    })?;
    let file_name = file_name.as_str();

    let source = std::fs::read_to_string(file_name).map_err(|e| {
        eprintln!("{e}");
        ErrorCode::FileNotFound
    })?;
    let source = source.as_str();

    match parse(source) {
        Ok(source) => {
            println!("{source:#?}\n");
            // println!("{source}");
            Ok(())
        }
        Err(e) => {
            for report in japp::nom_make_reports(file_name, &e) {
                if let Err(e) = report.print((file_name, ariadne::Source::from(source))) {
                    eprintln!("{e}");
                    return Err(ErrorCode::IoError);
                }
            }
            Err(ErrorCode::ParseError)
        }
    }
}
