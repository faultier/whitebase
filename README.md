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

use std::io::{BufferedReader, File};
use std::io::stdio::{stdin, stdout_raw};
use whitebase::machine::Interpreter;
use whitebase::syntax::Whitespace;

fn main() {
    match File::open(&Path::new("hello.ws")) {
        Ok(file) => {
            let mut buffer = BufferedReader::new(file);
            let ip = Interpreter::new(stdin(), stdout_raw(), Whitespace::new());
            match ip.run(&mut buffer) {
                Err(e) => fail!("{}", e),
                _ => (),
            }
        },
        Err(e) => fail!("{}", e),
    }
}
```

## License

This project distributed under the MIT License.
http://opensource.org/licenses/MIT
