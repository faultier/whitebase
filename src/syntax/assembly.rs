use std::collections::HashMap;
use std::io::{BufReader, EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::iter::count;

use bytecode::ByteCodeReader;
use syntax;
use syntax::{AST, Syntax};

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
pub struct Assembly;

impl Syntax for Assembly {
    fn new() -> Assembly { Assembly }

    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        self.parse(&mut BufReader::new(input.as_bytes()), output)
    }

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
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
                        "PUSH"     => Ok(syntax::WBPush(try_number!(val))),
                        "DUP"      => Ok(syntax::WBDuplicate),
                        "COPY"     => Ok(syntax::WBCopy(try_number!(val))),
                        "SWAP"     => Ok(syntax::WBSwap),
                        "DISCARD"  => Ok(syntax::WBDiscard),
                        "SLIDE"    => Ok(syntax::WBSlide(try_number!(val))),
                        "ADD"      => Ok(syntax::WBAddition),
                        "SUB"      => Ok(syntax::WBSubtraction),
                        "MUL"      => Ok(syntax::WBMultiplication),
                        "DIV"      => Ok(syntax::WBDivision),
                        "MOD"      => Ok(syntax::WBModulo),
                        "STORE"    => Ok(syntax::WBStore),
                        "RETRIEVE" => Ok(syntax::WBRetrieve),
                        "MARK"     => Ok(syntax::WBMark(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "CALL"     => Ok(syntax::WBCall(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "JUMP"     => Ok(syntax::WBJump(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "JUMPZ"    => Ok(syntax::WBJumpIfZero(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "JUMPN"    => Ok(syntax::WBJumpIfNegative(self.marker(val.to_string(), &mut labels, &mut counter))),
                        "RETURN"   => Ok(syntax::WBReturn),
                        "EXIT"     => Ok(syntax::WBExit),
                        "PUTC"     => Ok(syntax::WBPutCharactor),
                        "PUTN"     => Ok(syntax::WBPutNumber),
                        "GETC"     => Ok(syntax::WBGetCharactor),
                        "GETN"     => Ok(syntax::WBGetNumber),
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

    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.disassemble(input, &mut ast));
        for inst in ast.iter() {
            let res = match inst {
                &syntax::WBPush(n)              => write!(output, "PUSH {}\n", n),
                &syntax::WBDuplicate            => output.write_line("DUP"),
                &syntax::WBCopy(n)              => write!(output, "COPY {}\n", n),
                &syntax::WBSwap                 => output.write_line("SWAP"),
                &syntax::WBDiscard              => output.write_line("DISCARD"),
                &syntax::WBSlide(n)             => write!(output, "SLIDE {}\n", n),
                &syntax::WBAddition             => output.write_line("ADD"),
                &syntax::WBSubtraction          => output.write_line("SUB"),
                &syntax::WBMultiplication       => output.write_line("MUL"),
                &syntax::WBDivision             => output.write_line("DIV"),
                &syntax::WBModulo               => output.write_line("MOD"),
                &syntax::WBStore                => output.write_line("STORE"),
                &syntax::WBRetrieve             => output.write_line("RETRIEVE"),
                &syntax::WBMark(n)              => write!(output, "MARK {:X}\n", n),
                &syntax::WBCall(n)              => write!(output, "CALL {:X}\n", n),
                &syntax::WBJump(n)              => write!(output, "JUMP {:X}\n", n),
                &syntax::WBJumpIfZero(n)        => write!(output, "JUMPZ {:X}\n", n),
                &syntax::WBJumpIfNegative(n)    => write!(output, "JUMPN {:X}\n", n),
                &syntax::WBReturn               => output.write_line("RETURN"),
                &syntax::WBExit                 => output.write_line("EXIT"),
                &syntax::WBPutCharactor         => output.write_line("PUTC"),
                &syntax::WBPutNumber            => output.write_line("PUTN"),
                &syntax::WBGetCharactor         => output.write_line("GETC"),
                &syntax::WBGetNumber            => output.write_line("GETN"),
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
        let syntax: Assembly = Syntax::new();
        let mut ast: AST = vec!();
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
        let syntax: Assembly = Syntax::new();
        let mut ast: AST = vec!();
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
        let syntax: Assembly = Syntax::new();
        let mut ast: AST = vec!();
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
        let syntax: Assembly = Syntax::new();
        let mut ast: AST = vec!();
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
        let syntax: Assembly = Syntax::new();
        let mut ast: AST = vec!();
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
            let syntax: Assembly = Syntax::new();
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
