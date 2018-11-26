# loe

Very fast and yet another line ending (CRLF <-> LF) converter written in Rust. It is distributed both as a library and
as a runnable program. Designed with performance in mind (works on byte buffers instead of strings).

## Features

* CRLF -> LF and LF -> CRLF conversion
* Input encoding checking (Ascii, UTF-8, easily extensible)
* That's basically it

## Usage

### Command line

```shell
$ cargo install loe
$ loe --help  # prints usage
$ loe -o unix.txt dos.txt
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

## Some Benchmarks

The following measurement was performed very unprofessionaly but still it can reveal some information. Commands were
spawned on a mediocre personal laptop on Arch linux (kernel 4.19.2) and in these versions: loe - 0.2.0,
[dos2unix](http://dos2unix.sourceforge.net/) - 7.4.0. The file `bench.txt` was 117M file with ~160k lines with only
ascii characters in it (but utf-8 does not change the results significantly).

The argument `-e ascii` is used becase dos2unix by default checks if the file is not binary. This argument sort of
imitate this behavior.

* `loe -e ascii -o out.txt bench.txt` - 0.70s
* `dos2unix -n bench.txt out.txt` - 2.43s (dos2unix / loe ~= 3.5x)
* `loe -e ascii -n crlf -o out.txt bench.txt` - 0.76s
* `unix2dos -n bench.txt out.txt` - 2.87s (unix2dos / loe ~= 3.8x)
* `loe -e ascii -o bench.txt bench.txt` - 0.59s
* `dos2unix bench.txt` - 2.39s (dos2unix / loe ~= 4.05x)
* `tr -d '\r' < bench.txt > out.txt` - 0.12s (loe / tr ~= 5.8x)

Bear in mind that dos2unix offers more features than loe. On the other hand, implementing them in loe should not affect
the performance much.

If you do not agree how the measurement was performed, please let me know.

## License

loe is licensed under [MIT](LICENSE). Feel free to use it, contribute or spread the word.
