//! Assembler and Disassembler.

use std::collections::HashMap;
use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::iter::{Counter, count};

use bytecode::ByteCodeReader;
use ir;
use ir::Instruction;
use syntax::{Compiler, Decompiler};

macro_rules! try_number(
    ($val:expr) => (match from_str($val) {
        Some(n) => n,
        None => return Err(IoError {
            kind: InvalidInput,
            desc: "invalid value format",
            detail: Some(format!("expected number, but {}", $val)),
        }),
    })
)

/// Assembler and Disassembler.
pub struct Assembly;

impl Assembly {
    /// Create a new `Assembly`.
    pub fn new() -> Assembly { Assembly }

    fn marker(&self, label: String, labels: &mut HashMap<String, i64>, counter: &mut Counter<i64>) -> i64 {
        match labels.find_copy(&label) {
            Some(val) => val,
            None => {
                let val = counter.next().unwrap();
                labels.insert(label, val);
                val
            },
        }
    }
}

impl Compiler for Assembly {
    fn parse<B: Buffer>(&self, input: &mut B, output: &mut Vec<Instruction>) -> IoResult<()> {
        let mut labels = HashMap::new();
        let mut counter = count(1, 1);
        loop {
            let ret = match input.read_line() {
                Ok(line) => {
                    let inst = line.replace("\n","");
                    let slice = inst.as_slice();
                    if slice.len() == 0 { continue }
                    if slice.char_at(0) == ';' { continue }
                    let (mnemonic, val) = match slice.find(' ') {
                        Some(n) => (slice.slice_to(n), slice.slice_from(n + 1)),
                        None => (slice, ""),
                    };
                    match mnemonic {
                        "PUSH"     => Ok(ir::WBPush(try_number!(val))),
                        "DUP"      => Ok(ir::WBDuplicate),
                        "COPY"     => Ok(ir::WBCopy(try_number!(val))),
                        "SWAP"     => Ok(ir::WBSwap),
                        "DISCARD"  => Ok(ir::WBDiscard),
                        "SLIDE"    => Ok(ir::WBSlide(try_number!(val))),
                        "ADD"      => Ok(ir::WBAddition),
                        "SUB"      => Ok(ir::WBSubtraction),
                        "MUL"      => Ok(ir::WBMultiplication),
                        "DIV"      => Ok(ir::WBDivision),
                        "MOD"      => Ok(ir::WBModulo),
                        "STORE"    => Ok(ir::WBStore),
                        "RETRIEVE" => Ok(ir::WBRetrieve),
                        "MARK"     => Ok(ir::WBMark(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "CALL"     => Ok(ir::WBCall(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "JUMP"     => Ok(ir::WBJump(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "JUMPZ"    => Ok(ir::WBJumpIfZero(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "JUMPN"    => Ok(ir::WBJumpIfNegative(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "RETURN"   => Ok(ir::WBReturn),
                        "EXIT"     => Ok(ir::WBExit),
                        "PUTC"     => Ok(ir::WBPutCharactor),
                        "PUTN"     => Ok(ir::WBPutNumber),
                        "GETC"     => Ok(ir::WBGetCharactor),
                        "GETN"     => Ok(ir::WBGetNumber),
                        _          => Err(standard_error(InvalidInput)),
                    }
                },
                Err(e) => Err(e),
            };
            match ret {
                Ok(inst) => output.push(inst),
                Err(ref e) if e.kind == EndOfFile => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Decompiler for Assembly {
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.disassemble(input, &mut ast));
        for inst in ast.iter() {
            let res = match inst {
                &ir::WBPush(n)              => write!(output, "PUSH {}\n", n),
                &ir::WBDuplicate            => output.write_line("DUP"),
                &ir::WBCopy(n)              => write!(output, "COPY {}\n", n),
                &ir::WBSwap                 => output.write_line("SWAP"),
                &ir::WBDiscard              => output.write_line("DISCARD"),
                &ir::WBSlide(n)             => write!(output, "SLIDE {}\n", n),
                &ir::WBAddition             => output.write_line("ADD"),
                &ir::WBSubtraction          => output.write_line("SUB"),
                &ir::WBMultiplication       => output.write_line("MUL"),
                &ir::WBDivision             => output.write_line("DIV"),
                &ir::WBModulo               => output.write_line("MOD"),
                &ir::WBStore                => output.write_line("STORE"),
                &ir::WBRetrieve             => output.write_line("RETRIEVE"),
                &ir::WBMark(n)              => write!(output, "MARK {:X}\n", n),
                &ir::WBCall(n)              => write!(output, "CALL {:X}\n", n),
                &ir::WBJump(n)              => write!(output, "JUMP {:X}\n", n),
                &ir::WBJumpIfZero(n)        => write!(output, "JUMPZ {:X}\n", n),
                &ir::WBJumpIfNegative(n)    => write!(output, "JUMPN {:X}\n", n),
                &ir::WBReturn               => output.write_line("RETURN"),
                &ir::WBExit                 => output.write_line("EXIT"),
                &ir::WBPutCharactor         => output.write_line("PUTC"),
                &ir::WBPutNumber            => output.write_line("PUTN"),
                &ir::WBGetCharactor         => output.write_line("GETC"),
                &ir::WBGetNumber            => output.write_line("GETN"),
            };
            match res {
                Err(ref e) if e.kind == EndOfFile => break,
                Err(e) => return Err(e),
                _ => continue,
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::{MemReader, MemWriter};
    use std::str::from_utf8;
    use super::*;
    use ir::*;
    use bytecode::ByteCodeWriter;
    use syntax::*;

    #[test]
    fn test_parse_stack() {
        let source = vec!(
            "PUSH 1",
            "DUP",
            "COPY -1",
            "SWAP",
            "DISCARD",
            "SLIDE 1000",
            ).connect("\n");
        let syntax = Assembly::new();
        let mut ast: Vec<Instruction> = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(1)));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBCopy(-1)));
        assert_eq!(ast.shift(), Some(WBSwap));
        assert_eq!(ast.shift(), Some(WBDiscard));
        assert_eq!(ast.shift(), Some(WBSlide(1000)));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_arithmetic() {
        let source = vec!(
            "ADD",
            "SUB",
            "MUL",
            "DIV",
            "MOD",
            ).connect("\n");
        let syntax = Assembly::new();
        let mut ast: Vec<Instruction> = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBAddition));
        assert_eq!(ast.shift(), Some(WBSubtraction));
        assert_eq!(ast.shift(), Some(WBMultiplication));
        assert_eq!(ast.shift(), Some(WBDivision));
        assert_eq!(ast.shift(), Some(WBModulo));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_heap() {
        let source = vec!(
            "STORE",
            "RETRIEVE",
            ).connect("\n");
        let syntax = Assembly::new();
        let mut ast: Vec<Instruction> = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_flow() {
        let source = vec!(
            "MARK 1",
            "CALL string",
            "JUMP 1",
            "JUMPZ other",
            "JUMPN 1",
            "RETURN",
            "EXIT",
            ).connect("\n");
        let syntax = Assembly::new();
        let mut ast: Vec<Instruction> = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBMark(1)));
        assert_eq!(ast.shift(), Some(WBCall(2)));
        assert_eq!(ast.shift(), Some(WBJump(1)));
        assert_eq!(ast.shift(), Some(WBJumpIfZero(3)));
        assert_eq!(ast.shift(), Some(WBJumpIfNegative(1)));
        assert_eq!(ast.shift(), Some(WBReturn));
        assert_eq!(ast.shift(), Some(WBExit));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_io() {
        let source = vec!(
            "PUTC",
            "PUTN",
            "GETC",
            "GETN",
            ).connect("\n");
        let syntax = Assembly::new();
        let mut ast: Vec<Instruction> = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPutCharactor));
        assert_eq!(ast.shift(), Some(WBPutNumber));
        assert_eq!(ast.shift(), Some(WBGetCharactor));
        assert_eq!(ast.shift(), Some(WBGetNumber));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_generate() {
        let mut writer = MemWriter::new();
        {
            let mut bcw = MemWriter::new();
            bcw.write_push(1).unwrap();
            bcw.write_dup().unwrap();
            bcw.write_copy(2).unwrap();
            bcw.write_swap().unwrap();
            bcw.write_discard().unwrap();
            bcw.write_slide(3).unwrap();
            bcw.write_add().unwrap();
            bcw.write_sub().unwrap();
            bcw.write_mul().unwrap();
            bcw.write_div().unwrap();
            bcw.write_mod().unwrap();
            bcw.write_store().unwrap();
            bcw.write_retrieve().unwrap();
            bcw.write_mark(1).unwrap();
            bcw.write_call(15).unwrap();
            bcw.write_jump(2).unwrap();
            bcw.write_jumpz(16).unwrap();
            bcw.write_jumpn(32).unwrap();
            bcw.write_return().unwrap();
            bcw.write_exit().unwrap();
            bcw.write_putc().unwrap();
            bcw.write_putn().unwrap();
            bcw.write_getc().unwrap();
            bcw.write_getn().unwrap();
            let mut bcr = MemReader::new(bcw.unwrap());
            let syntax = Assembly::new();
            syntax.decompile(&mut bcr, &mut writer).unwrap();
        }
        let result = from_utf8(writer.get_ref()).unwrap();
        let expected = vec!(
            "PUSH 1", "DUP", "COPY 2", "SWAP", "DISCARD", "SLIDE 3",
            "ADD", "SUB", "MUL", "DIV", "MOD",
            "STORE", "RETRIEVE",
            "MARK 1", "CALL F", "JUMP 2", "JUMPZ 10", "JUMPN 20", "RETURN", "EXIT",
            "PUTC", "PUTN", "GETC", "GETN", ""
            ).connect("\n");
        assert_eq!(result, expected.as_slice());
    }
}
