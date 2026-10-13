//! Usage: `lwon-dump [-a] [-d] [file ...]`
//!
//! Read each file (or standard input, if there are none or the file is `-`)
//! as a sequence of LWON values and print each value in LWON format. With
//! `-a`, the values are implicit arrays; with `-d`, implicit dictionaries.
//! Exits with status 1 after printing a message to standard error if the
//! input is not valid LWON or a file cannot be read.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use lwon::{Parser, Value};

enum Implicit {
    None,
    Array,
    Dictionary,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut implicit = Implicit::None;
    let mut optind = 0;
    while optind < args.len() && args[optind].starts_with('-') && args[optind] != "-" {
        match args[optind].as_str() {
            "-a" => implicit = Implicit::Array,
            "-d" => implicit = Implicit::Dictionary,
            _ => {}
        }
        optind += 1;
    }
    let mut files = &args[optind..];
    let stdin = ["-".to_string()];
    if files.is_empty() {
        files = &stdin;
    }
    for filename in files {
        let (text, name) = match read(filename) {
            Ok(input) => input,
            Err(_) => {
                eprintln!("File not found: {filename}");
                return ExitCode::FAILURE;
            }
        };
        let mut parser = Parser::new(&text, name);
        loop {
            let value = match implicit {
                Implicit::None => parser.parse(),
                Implicit::Array => parser.parse_implicit_array().map(|a| a.map(Value::from)),
                Implicit::Dictionary => parser
                    .parse_implicit_dictionary()
                    .map(|d| d.map(Value::from)),
            };
            match value {
                Ok(Some(value)) => println!("{value}"),
                Ok(None) => break,
                Err(e) => {
                    let _ = io::stdout().flush();
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    ExitCode::SUCCESS
}

/// The contents of the named file, or of standard input if the name is "-",
/// and the name to use for it in error messages.
fn read(filename: &str) -> io::Result<(String, String)> {
    if filename == "-" {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;
        Ok((text, "stdin".to_string()))
    } else {
        Ok((std::fs::read_to_string(filename)?, filename.to_string()))
    }
}
