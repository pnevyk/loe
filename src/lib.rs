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
/// process(&mut input, &mut output, Config::default().encoding(Encoding::Ascii).transform(TransformMode::CRLF));
/// let actual = String::from_utf8(output.into_inner()).unwrap();
/// assert_eq!(actual, expected);
/// ```
pub struct Config<E: Into<Box<dyn EncodingChecker>>, T: Into<Box<dyn Transform>>> {
    encoding_checker: E,
    transform_mode: T,
}

impl Config<Encoding, TransformMode> {
    /// Creates default instance of the config (no encoding, UTF-8).
    pub fn new() -> Self {
        Config {
            encoding_checker: Encoding::Ignore,
            transform_mode: TransformMode::LF,
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

/// Helper trait which just returns the name of the encoding. It is used in error reporting.
pub trait RepresentsEncoding {
    fn name(&self) -> &str;
}

impl RepresentsEncoding for Encoding {
    fn name(&self) -> &str {
        match *self {
            Encoding::Utf8 => "UTF-8",
            Encoding::Ascii => "Ascii",
            Encoding::Ignore => "<none>",
        }
    }
}

/// Error which can occur during processing.
#[derive(Debug)]
pub enum ParseError {
    /// The input is in invalid encoding. This enum variant also holds the name of expected
    /// encodng.
    InvalidEncoding(String),
    /// An I/O error occured.
    IoError(io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidEncoding(ref encoding) => write!(
                f,
                "The file is not in expected encoding which is {}",
                encoding
            ),
            ParseError::IoError(ref err) => write!(f, "{}", err),
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
    E: Into<Box<dyn EncodingChecker>> + RepresentsEncoding,
    T: Into<Box<dyn Transform>>,
{
    let encoding_name = config.encoding_checker.name().to_string();
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
            .map_err(|err| ParseError::IoError(err))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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
}
