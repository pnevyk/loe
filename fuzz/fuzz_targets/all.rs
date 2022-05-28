#![allow(unused_must_use)]
#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate loe;

use std::io::Cursor;

use loe::{process, Config, Encoding, TransformMode};

fuzz_target!(|data: &[u8]| {
    let mut input = Cursor::new(data);
    let mut output = Cursor::new(Vec::<u8>::new());

    for enc in &[Encoding::Ignore, Encoding::Ascii, Encoding::Utf8] {
        for trans in &[TransformMode::Lf, TransformMode::Crlf] {
            let config = Config::default().encoding(*enc).transform(*trans);
            // do not call `unwrap` because we do not check for errors (io error, encoding error, etc.)
            process(&mut input, &mut output, config);
        }
    }
});
