# loe

Yet another line ending (CRLF <-> LF) converter written in Rust. It is distributed both as a library and as a runnable
program. Designed with performance in mind (works on byte buffers instead of strings).

## Features

* CRLF -> LF and LF -> CRLF conversion
* Input encoding checking (Ascii, UTF-8, easily extensible)

## Usage

### Command line

```shell
$ cargo install loe
$ loe --help  # prints usage
$ loe -o dos.txt unix.txt
```

### Library

In `Cargo.toml`

```toml
[dependencies]
loe = "0.2"
```

In your source file:

```rust
extern crate loe;

use std::io::Cursor;

use loe::{process, Config};

fn convert(input: String) -> String {
    let mut input = Cursor::new(input);
    let mut output = Cursor::new(Vec::new());

    process(&mut input, &mut output, Config::default());
    String::from_utf8(output.into_inner()).unwrap()
}
```

See [documentation](https://docs.rs/loe/) to know more!

## License

loe is licensed under [MIT](LICENSE). Feel free to use it, contribute or spread the word.
