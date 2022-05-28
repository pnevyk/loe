mod encodings;
mod transforms;

use std::fmt;
use std::io::{self, Read, Write};

pub use self::encodings::{Encoding, EncodingChecker};
pub use self::transforms::{Transform, TransformMode};

const BUFFER_SIZE: usize = 4096;

/// Configuration for processing. Two things can be set: encoding of input and type of line ending.
///
/// ```
/// use std::io::Cursor;
///
/// use loe::{process, Config, Encoding, TransformMode};
///
/// let mut input = Cursor::new("hello\nworld!\n");
/// let expected = "hello\r\nworld!\r\n";
/// let mut output = Cursor::new(Vec::new());
///
/// process(&mut input, &mut output, Config::default().encoding(Encoding::Ascii).transform(TransformMode::Crlf));
/// let actual = String::from_utf8(output.into_inner()).unwrap();
/// assert_eq!(actual, expected);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config<E: Into<Box<dyn EncodingChecker>>, T: Into<Box<dyn Transform>>> {
    encoding_checker: E,
    transform_mode: T,
}

impl Config<Encoding, TransformMode> {
    /// Creates default instance of the config (no encoding, UTF-8).
    pub fn new() -> Self {
        Config {
            encoding_checker: Encoding::Ignore,
            transform_mode: TransformMode::Lf,
        }
    }
}

impl<E: Into<Box<dyn EncodingChecker>>, T: Into<Box<dyn Transform>>> Config<E, T> {
    /// Changes the encoding. Given value must be a type which implements
    /// Into<Box<dyn EncodingChecker>>. For more info, see documentation for
    /// [EncodingChecker](trait.EncodingChecker.html).
    pub fn encoding(self, encoding: E) -> Self {
        Config {
            encoding_checker: encoding,
            ..self
        }
    }

    /// Changes the transformation. Given value must be a type which implements
    /// Into<Box<dyn Transform>>. For more info, see documentation for
    /// [Transform](trait.Transform.html).
    pub fn transform(self, transform: T) -> Self {
        Config {
            transform_mode: transform,
            ..self
        }
    }
}

impl Default for Config<Encoding, TransformMode> {
    fn default() -> Self {
        Config::new()
    }
}

/// Error which can occur during processing.
#[derive(Debug)]
pub enum ParseError {
    /// The input is in invalid encoding. This enum variant also holds the name of expected
    /// encoding.
    InvalidEncoding(String),
    /// An I/O error occurred.
    IoError(io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidEncoding(ref encoding) => {
                write!(f, "file is not in expected encoding '{}'", encoding)
            }
            ParseError::IoError(ref err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::InvalidEncoding(_) => None,
            ParseError::IoError(error) => Some(error),
        }
    }
}

/// The entry point of *loe*. It processes the given input and write the result into the given
/// output. Its behavior is dependent on given config.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use std::io::Cursor;
///
/// use loe::{process, Config};
///
/// let mut input = Cursor::new("hello\r\nworld!\r\n");
/// let expected = "hello\nworld!\n";
/// let mut output = Cursor::new(Vec::new());
///
/// process(&mut input, &mut output, Config::default());
/// let actual = String::from_utf8(output.into_inner()).unwrap();
/// assert_eq!(actual, expected);
/// ```
///
/// Wrapping the function to convert string to string:
///
/// ```
/// use std::io::Cursor;
///
/// use loe::{process, Config};
///
/// fn convert(input: String) -> String {
///     let mut input = Cursor::new(input);
///     let mut output = Cursor::new(Vec::new());
///
///     process(&mut input, &mut output, Config::default());
///     String::from_utf8(output.into_inner()).unwrap()
/// }
///
/// assert_eq!(convert("hello\r\nworld!\r\n".to_string()), "hello\nworld!\n".to_string());
/// ```
pub fn process<I, O, E, T>(
    input: &mut I,
    output: &mut O,
    config: Config<E, T>,
) -> Result<(), ParseError>
where
    I: Read,
    O: Write,
    E: Into<Box<dyn EncodingChecker>> + fmt::Display,
    T: Into<Box<dyn Transform>>,
{
    let encoding_name = format!("{}", config.encoding_checker);
    let mut encoding: Box<dyn EncodingChecker> = config.encoding_checker.into();
    let mut transform: Box<dyn Transform> = config.transform_mode.into();

    let mut read_buffer = [0; BUFFER_SIZE];
    let mut write_buffer = [0; 2 * BUFFER_SIZE];

    while let Ok(n) = input.read(&mut read_buffer) {
        if n == 0 {
            break;
        }

        let mut out_ptr = 0;
        for in_ptr in 0..n {
            if !encoding.feed(read_buffer[in_ptr]) {
                return Err(ParseError::InvalidEncoding(encoding_name));
            }
            out_ptr = transform.transform_buffer(in_ptr, out_ptr, &read_buffer, &mut write_buffer);
        }

        output
            .write(&write_buffer[0..out_ptr])
            .map_err(ParseError::IoError)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prop_assert, proptest, proptest_helper};
    use std::io::Cursor;

    const LF_BYTE: u8 = b'\n';
    const CR_BYTE: u8 = b'\r';

    #[test]
    fn basic() {
        let mut input = Cursor::new("hello\r\nworld!\r\n");
        let expected = "hello\nworld!\n";
        let mut output = Cursor::new(vec![0; expected.len()]);

        assert!(process(&mut input, &mut output, Config::default()).is_ok());
        let output = String::from_utf8(output.into_inner());
        assert!(output.is_ok());
        let output = output.unwrap();
        assert_eq!(output, expected);
    }

    fn filter(iterator: impl Iterator<Item = u8>) -> Vec<u8> {
        iterator
            .filter(|b| b != &LF_BYTE && b != &CR_BYTE)
            .collect()
    }

    proptest! {
        #[test]
        fn prop_lf(data in ".*") {
            let mut input = Cursor::new(data);
            let mut output = Cursor::new(Vec::<u8>::new());

            process(&mut input, &mut output, Config::default().transform(TransformMode::Lf)).unwrap();

            let output = output.into_inner();

            // no CR byte
            prop_assert!(!output.iter().any(|b| b == &CR_BYTE), "no CR byte");
        }

        #[test]
        fn prop_crlf(data in ".*") {
            let mut input = Cursor::new(data);
            let mut output = Cursor::new(Vec::<u8>::new());

            process(&mut input, &mut output, Config::default().transform(TransformMode::Crlf)).unwrap();

            let output = output.into_inner();

            // no LF byte
            prop_assert!(!output.iter().any(|b| b == &LF_BYTE), "no LF byte");
        }

        #[test]
        fn prop_inverse(data in "[^\\r]*") {
            let mut input = Cursor::new(data);
            let mut output = Cursor::new(Vec::<u8>::new());
            let mut output2 = Cursor::new(Vec::<u8>::new());

            process(&mut input, &mut output, Config::default().transform(TransformMode::Crlf)).unwrap();

            output.set_position(0);
            process(&mut output, &mut output2, Config::default().transform(TransformMode::Lf)).unwrap();

            let input = input.into_inner().bytes().collect::<Vec<_>>();
            let output = output2.into_inner();

            prop_assert!(input == output, "CRLF to LF and back");
        }


        #[test]
        fn prop_preserve_rest(data in ".*") {
            let input_filtered = filter(data.bytes());

            let mut input = Cursor::new(data);
            let mut output = Cursor::new(Vec::<u8>::new());
            let mut output2 = Cursor::new(Vec::<u8>::new());

            process(&mut input, &mut output, Config::default().transform(TransformMode::Lf)).unwrap();

            input.set_position(0);
            process(&mut input, &mut output2, Config::default().transform(TransformMode::Crlf)).unwrap();

            let output_filtered = filter(output.into_inner().into_iter());
            prop_assert!(input_filtered == output_filtered, "the rest is preserved (LF)");

            let output_filtered = filter(output2.into_inner().into_iter());
            prop_assert!(input_filtered == output_filtered, "the rest is preserved (CRLF)");
        }
    }
}
