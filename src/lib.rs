/*! The infrastructure for implementing esolang.

`whitebase` provides the virtual machine,
parsers and generators, and assembly language.

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
*/

#![crate_name="whitebase"]
#![crate_type="rlib"]
#![warn(missing_doc)]
#![feature(phase, globs, macro_rules)]
#![experimental]

#[phase(plugin, link)] extern crate log;

pub static VERSION_MAJOR: uint = 0;
pub static VERSION_MINOR: uint = 1;
pub static VERSION_TINY: uint = 0;
pub static PRE_RELEASE: bool = true;

/// Build version string.
pub fn version() -> String {
    format!("{}.{}.{}{}",
            VERSION_MAJOR, VERSION_MINOR, VERSION_TINY,
            if PRE_RELEASE { "-pre" } else { "" })
}

pub mod bytecode;
pub mod ir;
pub mod machine;
pub mod syntax;
