# Whitebase

This project provides infrastructure for implementing esolang.

- The virtual machine having the instruction set based on [Whitespace](http://compsoc.dur.ac.uk/whitespace/index.php)'s specification.
- Parsers and code generators for some languages (e.g. [Whitespace](http://compsoc.dur.ac.uk/whitespace/index.php), [Ook!](http://www.dangermouse.net/esoteric/ook.html), etc)
- Simple assembly language

## Prerequisites

- [Rust](http://www.rust-lang.org/) v0.12.0.

## Usage

### Compile and execute

```rust
extern crate whitebase;

use std::io::{BufferedReader, File, MemReader, MemWriter};
use std::io::stdio::{stdin, stdout_raw};
use whitebase::machine::Machine;
use whitebase::syntax::{Syntax, Whitespace};

fn main() {
    match File::open(&Path::new("hello.ws")) {
        Ok(file) => {
            let mut buffer = BufferedReader::new(file);
            let mut writer = MemWriter::new();
            let ws = Whitespace::new();
            match ws.compile(&mut buffer, &mut writer) {
                Err(e) => fail!("{}", e),
                _ => {
                    let mut reader = MemReader::new(writer.unwrap());
                    let mut machine = Machine::new(stdin(), stdout_raw());
                    match machine.run(&mut reader) {
                        Err(e) => fail!("{}", e),
                        _ => (),
                    }
                },
            }
        },
        Err(e) => fail!("{}", e),
    }
}
```

## License

This project distributed under the MIT License.
http://opensource.org/licenses/MIT
