//! This module provides encoding checkers if one wants to know if a correctly-encoded text file is
//! processed. This may be useful for checking if the processed file is not a binary file.
//!
//! # Examples
//!
//! ```
//! use std::io::Cursor;
//!
//! use loe::{process, Config, Encoding};
//!
//! // notice "ě" character which is not in ascii
//! let mut input = Cursor::new("ahoj\r\nsvěte!\r\n");
//! let mut output = Cursor::new(Vec::new());
//!
//! let processed = process(&mut input, &mut output, Config::default().encoding(Encoding::Ascii));
//! assert!(processed.is_err());
//! ```

use std::fmt;

/// Enumeration of core-supported encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// Special flag which disables any encoding checking on the input.
    Ignore,
    /// Ascii encoding, that is, each byte has to be less than 128.
    Ascii,
    /// Valid UTF-8 encoding.
    Utf8,
}

impl From<Encoding> for Box<dyn EncodingChecker> {
    fn from(val: Encoding) -> Self {
        match val {
            Encoding::Ignore => Box::new(Ignore::new()),
            Encoding::Ascii => Box::new(Ascii::new()),
            Encoding::Utf8 => Box::new(Utf8::new()),
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Encoding::Utf8 => "UTF-8",
            Encoding::Ascii => "Ascii",
            Encoding::Ignore => "<none>",
        };

        write!(f, "{}", name)
    }
}

/// Trait used for encoding checking. It should behave like a state machine to which bytes are fed.
/// If the passed bytes causes the checker to enter an invalid state, the method should return
/// false as the indication.
pub trait EncodingChecker {
    /// The only method of the checker. It gets the current byte of the input and returns if it is
    /// still valid encoding.
    fn feed(&mut self, byte: u8) -> bool;
}

struct Ignore;

impl Ignore {
    fn new() -> Self {
        Ignore
    }
}

impl EncodingChecker for Ignore {
    fn feed(&mut self, _byte: u8) -> bool {
        true
    }
}

struct Ascii;

impl Ascii {
    fn new() -> Self {
        Ascii
    }
}

impl EncodingChecker for Ascii {
    fn feed(&mut self, byte: u8) -> bool {
        byte < 128
    }
}

struct Utf8 {
    counter: Option<u8>,
}

impl Utf8 {
    fn new() -> Self {
        Utf8 { counter: None }
    }
}

impl EncodingChecker for Utf8 {
    fn feed(&mut self, byte: u8) -> bool {
        let counter = match self.counter {
            Some(counter) => {
                if byte & 0xc0 == 0x80 {
                    if counter == 0 {
                        None
                    } else {
                        Some(counter - 1)
                    }
                } else {
                    return false;
                }
            }
            None => {
                if byte & 0x80 == 0 {
                    None
                } else if byte & 0xe0 == 0xc0 {
                    Some(0)
                } else if byte & 0xf0 == 0xe0 {
                    Some(1)
                } else if byte & 0xf8 == 0xf0 {
                    Some(2)
                } else {
                    return false;
                }
            }
        };

        self.counter = counter;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_valid(encoding: &mut dyn EncodingChecker, bytes: &[u8]) {
        for byte in bytes {
            assert!(encoding.feed(*byte));
        }
    }

    fn feed_invalid(encoding: &mut dyn EncodingChecker, bytes: &[u8]) {
        let mut flag = true;
        for byte in bytes {
            flag &= encoding.feed(*byte);
        }
        assert!(!flag);
    }

    #[test]
    fn ascii() {
        feed_valid(&mut Ascii::new(), b"Hello world!");
        feed_valid(&mut Ascii::new(), &[0, 0x7f]);

        feed_invalid(&mut Ascii::new(), "Ahoj světe!".as_bytes());
    }

    #[test]
    fn utf8() {
        feed_valid(&mut Utf8::new(), b"Hello world!");
        feed_valid(&mut Utf8::new(), &[0, 0x7f]);
        feed_valid(&mut Utf8::new(), "Ahoj světe!".as_bytes());
        feed_valid(
            &mut Utf8::new(),
            &[0xc0, 0x80, 0xe0, 0x80, 0x80, 0xf0, 0x80, 0x80, 0x80],
        );
        feed_valid(
            &mut Utf8::new(),
            &[0xc0, 0xbf, 0xe0, 0xbf, 0xbf, 0xf0, 0xbf, 0xbf, 0xbf],
        );

        feed_invalid(&mut Utf8::new(), &[0x80]);
        feed_invalid(&mut Utf8::new(), &[0xc0, 0x7f]);
        feed_invalid(&mut Utf8::new(), &[0xc0, 0x80, 0x80]);
    }
}
