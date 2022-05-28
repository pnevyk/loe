#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate loe;

use std::io::Cursor;

use loe::{process, Config, Encoding, TransformMode};

fuzz_target!(|data: &[u8]| {
    // only utf8 strings
    if let Ok(s) = std::str::from_utf8(data) {
        let mut input = Cursor::new(s);
        let mut output = Cursor::new(Vec::<u8>::new());

        for trans in &[TransformMode::Lf, TransformMode::Crlf] {
            let config = Config::default().encoding(Encoding::Utf8).transform(*trans);
            process(&mut input, &mut output, config).unwrap();
        }
    }
});
