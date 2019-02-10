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

## Benchmarks

The benchmark was performed with [hyperfine](https://github.com/sharkdp/hyperfine) tool using `hyperfine '<command>'
--prepare 'sync; echo 3 | sudo tee /proc/sys/vm/drop_caches'` command. As dos2unix checks if the file is binary by
default, `-e ascii` is used for loe commands as it imitates this behavior in loe.

#### Hardware & software configuration

| Key | Value |
|:---|:---|
| File `bench.txt` | 144 MB, 200k lines, ascii only |
| Processor | Intel(R) Core(TM) i5-4200M CPU @ 2.50GHz |
| Disk | Samsung SSD 840 |
| Kernel | 4.20.5-arch1-1-ARCH |
| loe | 0.2.0 |
| dos2unix | 7.4.0 |
| hyperfine | 1.3.0 |

#### Results

| Command | Mean [ms] | Min…Max [ms] |
|:---|---:|---:|
| `loe -o out.txt bench.txt` | 1011.2 ± 59.9 | 968.5…1153.3 |
| `loe -e ascii -o out.txt bench.txt` | 962.9 ± 14.2 | 950.3…996.6 |
| `dos2unix -n bench.txt out.txt` | 1358.8 ± 94.1 | 1214.3…1502.9 |
| `loe -e ascii -n crlf -o out.txt bench.txt` | 1105.6 ± 18.0 | 1077.4…1135.2 |
| `unix2dos -n bench.txt out.txt` | 1763.6 ± 118.5 | 1636.4…1985.2 |
| `loe -e ascii -o bench.txt bench.txt` | 1086.4 ± 20.3 | 1050.0…1129.2 |
| `dos2unix bench.txt` | 1354.0 ± 81.1 | 1246.4…1484.0 |
| `tr -d '\r' < bench.txt > out.txt` | 472.4 ± 4.3 | 466.8…478.9 |

On average, loe is ~1.4 times faster than dos2unix. Bear in mind that dos2unix offers more features than loe. On the
other hand, implementing them in loe should not affect the performance much.

## License

loe is licensed under [MIT](LICENSE). Feel free to use it, contribute or spread the word.
