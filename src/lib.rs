#![crate_name="whitebase"]
#![crate_type="rlib"]

#![feature(phase, globs, macro_rules)]
#[phase(plugin, link)] extern crate log;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

pub mod bytecode;
pub mod syntax;
pub mod machine;
