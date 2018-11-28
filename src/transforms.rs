//! This module provides line endings transforms which is the cornerstone of this library.
//!
//! # Examples
//!
//! ```
//! use std::io::Cursor;
//!
//! use loe::{process, Config, TransformMode};
//!
//! let mut input = Cursor::new("hello\nworld!\n");
//! let expected = "hello\r\nworld!\r\n";
//! let mut output = Cursor::new(Vec::new());
//!
//! process(&mut input, &mut output, Config::default().transform(TransformMode::CRLF));
//! let actual = String::from_utf8(output.into_inner()).unwrap();
//! assert_eq!(actual, expected);
//! ```

const LF_CHAR: u8 = 0x0a;
const CR_CHAR: u8 = 0x0d;

/// Enumeration of possible transforms.
pub enum TransformMode {
    /// Windows line ending.
    CRLF,
    /// Unix line ending.
    LF,
}

impl Into<Box<dyn Transform>> for TransformMode {
    fn into(self) -> Box<dyn Transform> {
        match self {
            TransformMode::CRLF => Box::new(CRLF::new()),
            TransformMode::LF => Box::new(LF::new()),
        }
    }
}

/// Trait used for transformation of the input. It works on buffers due to the memory consumption
/// and performance reasons.
pub trait Transform {
    /// Transforms the input to the output. Most of the time it just copies the byte at index
    /// `in_ptr` from the input to the index `out_ptr` of the output. The function also returns the
    /// position of the first next writable byte in the output buffer, that is, one behind the last
    /// written one.
    fn transform_buffer(
        &mut self,
        in_ptr: usize,
        out_ptr: usize,
        input: &[u8],
        output: &mut [u8],
    ) -> usize;
}

struct CRLF;

impl CRLF {
    fn new() -> Self {
        CRLF
    }
}

impl Transform for CRLF {
    fn transform_buffer(
        &mut self,
        in_ptr: usize,
        mut out_ptr: usize,
        input: &[u8],
        output: &mut [u8],
    ) -> usize {
        if input[in_ptr] != CR_CHAR {
            if input[in_ptr] == LF_CHAR {
                output[out_ptr] = CR_CHAR;
                out_ptr += 1;
            }

            output[out_ptr] = input[in_ptr];
            out_ptr += 1;
        }

        out_ptr
    }
}

struct LF;

impl LF {
    fn new() -> Self {
        LF
    }
}

impl Transform for LF {
    fn transform_buffer(
        &mut self,
        in_ptr: usize,
        mut out_ptr: usize,
        input: &[u8],
        output: &mut [u8],
    ) -> usize {
        if input[in_ptr] != CR_CHAR {
            output[out_ptr] = input[in_ptr];
            out_ptr += 1;
        }

        out_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(transform: &mut dyn Transform, input: &[u8], expected: &[u8]) {
        let mut output = vec![0; input.len() * 2];

        let mut out_ptr = 0;
        for in_ptr in 0..input.len() {
            out_ptr = transform.transform_buffer(in_ptr, out_ptr, input, &mut output);
        }

        assert_eq!(out_ptr, expected.len());
        assert_eq!(&output[0..out_ptr], expected);
    }

    #[test]
    fn crlf_basic() {
        test(&mut CRLF::new(), b"Hello\nworld!\n", b"Hello\r\nworld!\r\n");
        test(
            &mut CRLF::new(),
            b"Hello\r\nworld!\r\n",
            b"Hello\r\nworld!\r\n",
        );
    }

    #[test]
    fn lf_basic() {
        test(&mut LF::new(), b"Hello\r\nworld!\r\n", b"Hello\nworld!\n");
        test(&mut LF::new(), b"Hello\nworld!\n", b"Hello\nworld!\n");
    }
}
