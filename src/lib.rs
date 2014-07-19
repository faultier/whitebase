/*! The infrastructure for implementing esolang.

`whitebase` provides the virtual machine,
parsers and generators, and assembly language.

```rust
extern crate whitebase;

use std::io::{BufReader, MemReader, MemWriter};
use whitebase::machine;
use whitebase::syntax::{Compile, Whitespace};

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

pub mod bytecode;
pub mod ir;
pub mod machine;
pub mod syntax;
