extern crate clap;
extern crate loe;

use std::env;
use std::fmt;
use std::fs::{self, File};
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
        .version("0.3.0")
        .about("Changes line endings to LF or CRLF")
        .author("Petr Nevyhoštěný")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Specifies a filepath where the transformed file is written to. If it is identical to the input filepath, the content is safely replaced (no data loss).")
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

    let input_path = matches.value_of("FILE").unwrap();
    let mut input = File::open(&input_path).unwrap_or_else(|err| print_error_and_exit(err));

    let default_output = format!("{}.out", input_path);
    let output_path_candidate = matches.value_of("output").unwrap_or(&default_output);

    let mut tmp = env::temp_dir();
    let (output_path, identical) = if output_path_candidate == input_path {
        tmp.push(default_output);
        (
            tmp.as_path()
                .to_str()
                .unwrap_or_else(|| print_error_and_exit("Input filepath is not valid utf-8")),
            true,
        )
    } else {
        (output_path_candidate, false)
    };

    let mut output = File::create(&output_path).unwrap_or_else(|err| print_error_and_exit(err));

    let encoding = matches
        .value_of("encoding")
        .map(|e| match e {
            "utf8" => Encoding::Utf8,
            "ascii" => Encoding::Ascii,
            _ => unreachable!(),
        })
        .unwrap_or(Encoding::Ignore);

    let transform = matches
        .value_of("ending")
        .map(|e| match e {
            "lf" => TransformMode::Lf,
            "crlf" => TransformMode::Crlf,
            _ => unreachable!(),
        })
        .unwrap();

    process(
        &mut input,
        &mut output,
        Config::default().encoding(encoding).transform(transform),
    )
    .unwrap_or_else(|err| print_error_and_exit(err));

    if identical {
        fs::copy(output_path, input_path).unwrap_or_else(|err| print_error_and_exit(err));
        fs::remove_file(output_path).unwrap_or_else(|err| print_error_and_exit(err));
    }
}
