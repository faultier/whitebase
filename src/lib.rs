/*! Infrastructure for implementing esolang.

`whitebase` provides the virtual machine,
parsers and generators, and assembly language.

# Examples

* Compile and execute

```rust
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
```
*/

#![crate_name="whitebase"]
#![crate_type="rlib"]

#![feature(phase, globs, macro_rules)]
#[phase(plugin, link)] extern crate log;

pub mod bytecode;
pub mod ir;
pub mod machine;
pub mod syntax;
