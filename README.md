# Whitebase
[![Build Status](https://travis-ci.org/faultier/rust-whitebase.svg?branch=master)](https://travis-ci.org/faultier/rust-whitebase)

This project provides infrastructure for implementing esolang.

## Features

- The virtual machine having the instruction set based on [Whitespace](http://compsoc.dur.ac.uk/whitespace/index.php)'s specification.
- Parsers and code generators for some languages (e.g. [Whitespace](http://compsoc.dur.ac.uk/whitespace/index.php), [Ook!](http://www.dangermouse.net/esoteric/ook.html), etc.)
- Simple assembly language

[Rust](http://www.rust-lang.org/) v0.12.0-pre support.

## Usage

### Compile and execute

```rust
extern crate whitebase;

use std::io::{BufReader, MemReader, MemWriter};
use whitebase::machine;
use whitebase::syntax::{Compiler, Whitespace};

fn main() {
    let src = "   \t\t \t  \t\n   \t  \t   \n\t\n  \t\n  \n\n\n";
    let mut buffer = BufReader::new(src.as_bytes());
    let mut writer = MemWriter::new();
    let ws = Whitespace::new();
    match ws.compile(&mut buffer, &mut writer) {
        Err(e) => fail!("{}", e),
        _ => {
            let mut reader = MemReader::new(writer.unwrap());
            let mut machine = machine::with_stdio();
            match machine.run(&mut reader) {
                Err(e) => fail!("{}", e),
                _ => (),
            }
        },
    }
}
```

## Application using Whitebase

- [Albino](https://github.com/faultier/rust-albino)

## License

This project distributed under the MIT License.
http://opensource.org/licenses/MIT
