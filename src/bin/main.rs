extern crate clap;
extern crate loe;

use std::fmt;
use std::fs::File;
use std::process;

use clap::{App, Arg};
use loe::{process, Config, Encoding, TransformMode};
use yansi::Paint;

fn print_error_and_exit<T: fmt::Display>(message: T) -> ! {
    eprintln!("{} {}", Paint::red("error:"), message);
    process::exit(1);
}

fn main() {
    let matches = App::new("loe")
        .version("0.1.0")
        .about("Changes line endings to LF or CRLF")
        .author("Petr Nevyhoštěný")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Specifies a filename where the transformed file is written to.")
                .takes_value(true),
        ).arg(Arg::with_name("FILE").help("Input file.").required(true))
        .arg(
            Arg::with_name("encoding")
                .short("e")
                .long("encoding")
                .help("Enables checking of encoding in the input file. By default, no checks are performed.")
                .takes_value(true)
                .possible_values(&["utf8", "ascii"])
                .value_name("utf8|ascii"),
        ).arg(
            Arg::with_name("ending")
                .short("n")
                .long("ending")
                .help("Specifies what line ending sequence is used.")
                .takes_value(true)
                .possible_values(&["lf", "crlf"])
                .value_name("lf|crlf")
                .default_value("lf"),
        ).get_matches();

    let filepath = matches.value_of("FILE").unwrap();
    let mut input = File::open(&filepath).unwrap_or_else(|err| print_error_and_exit(err));
    let mut output = File::create(
        &matches
            .value_of("output")
            .unwrap_or(format!("{}.out", filepath).as_str()),
    ).unwrap_or_else(|err| print_error_and_exit(err));

    let encoding = matches
        .value_of("encoding")
        .map(|e| match e {
            "utf8" => Encoding::Utf8,
            "ascii" => Encoding::Ascii,
            _ => unreachable!(),
        }).unwrap_or(Encoding::Ignore);

    let transform = matches
        .value_of("ending")
        .map(|e| match e {
            "lf" => TransformMode::LF,
            "crlf" => TransformMode::CRLF,
            _ => unreachable!(),
        }).unwrap();

    process(
        &mut input,
        &mut output,
        Config::default().encoding(encoding).transform(transform),
    ).unwrap_or_else(|err| print_error_and_exit(err));
}
