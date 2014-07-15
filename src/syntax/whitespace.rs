use std::collections::HashMap;
use std::io::{BufReader, EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::iter::{Counter, count};
use std::num::from_str_radix;

use bytecode::ByteCodeReader;
use syntax;
use syntax::{AST, Syntax};

macro_rules! write_num (
    ($w:expr, $cmd:expr, $n:expr) => (
        write!($w, "{}{}", $cmd,
               (if $n < 0 {
                   format!("\t{:t}\n", $n*-1)
               } else {
                   format!(" {:t}\n", $n)
               }).replace("0"," ").replace("1","\t")
        )
    )
)

pub struct Whitespace;

impl Whitespace {
    fn read_char<B: Buffer>(&self, source: &mut B) -> IoResult<char> {
        loop {
            match source.read_char() {
                Ok(c) if c == ' ' || c == '\n' || c == '\t' => return Ok(c),
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
    }

    fn parse_value<B: Buffer>(&self, input: &mut B) -> IoResult<String> {
        let mut value = String::new();
        loop {
            match self.read_char(input) {
                Ok(' ') => value.push_char('0'),
                Ok('\t') => value.push_char('1'),
                Ok('\n') => break,
                Ok(_) => continue,
                Err(ref e) if e.kind == EndOfFile => return Err(IoError {
                    kind: InvalidInput,
                    desc: "invalid value format",
                    detail: Some("no value terminator".to_string()),
                }),
                Err(e) => return Err(e),
            }
        }
        Ok(value)
    }

    fn parse_sign<B: Buffer>(&self, input: &mut B) -> IoResult<i64> {
        loop {
            match self.read_char(input) {
                Ok(' ') => return Ok(1),
                Ok('\t') => return Ok(-1),
                Ok('\n') => return Err(IoError {
                    kind: InvalidInput,
                    desc: "invalid value format",
                    detail: Some("no sign".to_string()),
                }),
                Ok(_) => continue,
                Err(ref e) if e.kind == EndOfFile => return Err(IoError {
                    kind: InvalidInput,
                    desc: "invalid value format",
                    detail: Some("no sign".to_string()),
                }),
                Err(e) => return Err(e),
            }
        }
    }

    fn parse_number<B: Buffer>(&self, input: &mut B) -> IoResult<i64> {
        let sign = try!(self.parse_sign(input));
        let val = try!(self.parse_value(input));
        match from_str_radix::<i64>(val.as_slice(), 2) {
            Some(n) => Ok(n*sign),
            None => Err(standard_error(InvalidInput)),
        }
    }

    fn parse_label<B: Buffer>(&self, input: &mut B, labels: &mut HashMap<String, i64>, counter: &mut Counter<i64>) -> IoResult<i64> {
        let label = try!(self.parse_value(input));
        Ok(self.marker(label, labels, counter))
    }
}

impl Syntax for Whitespace {
    fn new() -> Whitespace { Whitespace }

    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        self.parse(&mut BufReader::new(input.as_bytes()), output)
    }

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
        let mut labels = HashMap::new();
        let mut counter = count(1, 1);
        loop {
            let ret = match self.read_char(input) {
                Ok(' ') => match self.read_char(input) { // stack
                    Ok(' ') => Ok(syntax::WBPush(try!(self.parse_number(input)))),
                    Ok('\n') => match self.read_char(input) {
                        Ok(' ') => Ok(syntax::WBDuplicate),
                        Ok('\t') => Ok(syntax::WBSwap),
                        Ok('\n') => Ok(syntax::WBDiscard),
                        _ => Err(standard_error(InvalidInput)),
                    },
                    Ok('\t') => match self.read_char(input) {
                        Ok(' ') => Ok(syntax::WBCopy(try!(self.parse_number(input)))),
                        Ok('\n') => Ok(syntax::WBSlide(try!(self.parse_number(input)))),
                        Ok('\t') => Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("\"STT\" is unknown token".to_string()),
                        }),
                        _ => Err(standard_error(InvalidInput)),
                    },
                    _ => Err(standard_error(InvalidInput)),
                },
                Ok('\t') => match self.read_char(input) {
                    Ok(' ') => match self.read_char(input) { // arithmetic
                        Ok(' ') => match self.read_char(input) {
                            Ok(' ') => Ok(syntax::WBAddition),
                            Ok('\t') => Ok(syntax::WBSubtraction),
                            Ok('\n') => Ok(syntax::WBMultiplication),
                            _ => Err(standard_error(InvalidInput)),
                        },
                        Ok('\t') => match self.read_char(input) {
                            Ok(' ') => Ok(syntax::WBDivision),
                            Ok('\t') => Ok(syntax::WBModulo),
                            _ => Err(standard_error(InvalidInput)),
                        },
                        Err(e) => Err(e),
                        _ => Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("\"TSL\" is unknown token".to_string()),
                        }),
                    },
                    Ok('\t') => match self.read_char(input) { // heap
                        Ok(' ') => Ok(syntax::WBStore),
                        Ok('\t') => Ok(syntax::WBRetrieve),
                        Err(e) => Err(e),
                        _ => Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("\"TTL\" is unknown token".to_string()),
                        }),
                    },
                    Ok('\n') => match self.read_char(input) { // io
                        Ok(' ') => match self.read_char(input) {
                            Ok(' ') => Ok(syntax::WBPutCharactor),
                            Ok('\t') => Ok(syntax::WBPutNumber),
                            Err(e) => Err(e),
                            _ => Err(IoError {
                                kind: InvalidInput,
                                desc: "syntax error",
                                detail: Some("\"TLSL\" is unknown token".to_string()),
                            }),
                        },
                        Ok('\t') => match self.read_char(input) {
                            Ok(' ') => Ok(syntax::WBGetCharactor),
                            Ok('\t') => Ok(syntax::WBGetNumber),
                            Err(e) => Err(e),
                            _ => Err(IoError {
                                kind: InvalidInput,
                                desc: "syntax error",
                                detail: Some("\"TLTL\" is unknown token".to_string()),
                            }),
                        },
                        Err(e) => Err(e),
                        _ => Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("\"TLL\" is unknown token".to_string()),
                        }),
                    },
                    _ => Err(standard_error(InvalidInput)),
                },
                Ok('\n') => match self.read_char(input) { // flow
                    Ok(' ') => match self.read_char(input) {
                        Ok(' ') => Ok(syntax::WBMark(try!(self.parse_label(input, &mut labels, &mut counter)))),
                        Ok('\t') => Ok(syntax::WBCall(try!(self.parse_label(input, &mut labels, &mut counter)))),
                        Ok('\n') => Ok(syntax::WBJump(try!(self.parse_label(input, &mut labels, &mut counter)))),
                        Err(e) => Err(e),
                        _ => Err(standard_error(InvalidInput)),
                    },
                    Ok('\t') => match self.read_char(input) {
                        Ok(' ') => Ok(syntax::WBJumpIfZero(try!(self.parse_label(input, &mut labels, &mut counter)))),
                        Ok('\t') => Ok(syntax::WBJumpIfNegative(try!(self.parse_label(input, &mut labels, &mut counter)))),
                        Ok('\n') => Ok(syntax::WBReturn),
                        Err(e) => Err(e),
                        _ => Err(standard_error(InvalidInput)),
                    },
                    Ok('\n') => match self.read_char(input) {
                        Ok('\n') => Ok(syntax::WBExit),
                        Err(e) => Err(e),
                        Ok(' ') => Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("\"NNS\" is unknown token".to_string()),
                        }),
                        _ => Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("\"NNT\" is unknown token".to_string()),
                        }),
                    },
                    _ => Err(standard_error(InvalidInput)),
                },
                Err(e) => Err(e),
                _ => Err(standard_error(InvalidInput)),
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
            let ret = match inst {
                &syntax::WBPush(n)              => write_num!(output, "  ", n),
                &syntax::WBDuplicate            => write!(output, " \n "),
                &syntax::WBCopy(n)              => write_num!(output, " \t ", n),
                &syntax::WBSwap                 => write!(output, " \n\t"),
                &syntax::WBDiscard              => write!(output, " \n\n"),
                &syntax::WBSlide(n)             => write_num!(output, " \t\n", n),
                &syntax::WBAddition             => write!(output, "\t   "),
                &syntax::WBSubtraction          => write!(output, "\t  \t"),
                &syntax::WBMultiplication       => write!(output, "\t  \n"),
                &syntax::WBDivision             => write!(output, "\t \t "),
                &syntax::WBModulo               => write!(output, "\t \t\t"),
                &syntax::WBStore                => write!(output, "\t\t "),
                &syntax::WBRetrieve             => write!(output, "\t\t\t"),
                &syntax::WBMark(n)              => write_num!(output, "\n  ", n),
                &syntax::WBCall(n)              => write_num!(output, "\n \t", n),
                &syntax::WBJump(n)              => write_num!(output, "\n \n", n),
                &syntax::WBJumpIfZero(n)        => write_num!(output, "\n\t ", n),
                &syntax::WBJumpIfNegative(n)    => write_num!(output, "\n\t\t", n),
                &syntax::WBReturn               => write!(output, "\n\t\n"),
                &syntax::WBExit                 => write!(output, "\n\n\n"),
                &syntax::WBPutCharactor         => write!(output, "\t\n  "),
                &syntax::WBPutNumber            => write!(output, "\t\n \t"),
                &syntax::WBGetCharactor         => write!(output, "\t\n\t "),
                &syntax::WBGetNumber            => write!(output, "\t\n\t\t"),
            };
            match ret {
                Err(e) => return Err(e),
                _ => continue,
            }
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
            "   \t\n",      // PUSH 1
            " \n ",         // DUP
            " \t  \t\n",    // COPY 1
            " \n\t",        // SWAP
            " \n\n",        // DISCARD
            " \t\n \t\n",   // SLIDE 1
            ).concat();
        let syntax: Whitespace = Syntax::new();
        let mut ast: AST = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(1)));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBCopy(1)));
        assert_eq!(ast.shift(), Some(WBSwap));
        assert_eq!(ast.shift(), Some(WBDiscard));
        assert_eq!(ast.shift(), Some(WBSlide(1)));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_arithmetic() {
        let source = vec!(
            "\t   ",    // ADD
            "\t  \t",   // SUB
            "\t  \n",   // MUL
            "\t \t ",   // DIV
            "\t \t\t",  // MOD
            ).concat();
        let syntax: Whitespace = Syntax::new();
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
            "\t\t ",    // STORE
            "\t\t\t",   // RETRIEVE
            ).concat();
        let syntax: Whitespace = Syntax::new();
        let mut ast: AST = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_flow() {
        let source = vec!(
            "\n   \t\n",    // MARK 01
            "\n \t\t \n",   // CALL 10
            "\n \n \t\n",   // JUMP 01
            "\n\t \t \n",   // JUMPZ 10
            "\n\t\t \t\n",  // JUMPN 01
            "\n\t\n",       // RETURN
            "\n\n\n",       // EXIT
            ).concat();
        let syntax: Whitespace = Syntax::new();
        let mut ast: AST = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBMark(1)));
        assert_eq!(ast.shift(), Some(WBCall(2)));
        assert_eq!(ast.shift(), Some(WBJump(1)));
        assert_eq!(ast.shift(), Some(WBJumpIfZero(2)));
        assert_eq!(ast.shift(), Some(WBJumpIfNegative(1)));
        assert_eq!(ast.shift(), Some(WBReturn));
        assert_eq!(ast.shift(), Some(WBExit));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_io() {
        let source = vec!(
            "\t\n  ",   // PUTC
            "\t\n \t",  // PUTN
            "\t\n\t ",  // GETC
            "\t\n\t\t", // GETN
            ).concat();
        let syntax: Whitespace = Syntax::new();
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
            bcw.write_call(1).unwrap();
            bcw.write_jump(1).unwrap();
            bcw.write_jumpz(1).unwrap();
            bcw.write_jumpn(1).unwrap();
            bcw.write_return().unwrap();
            bcw.write_exit().unwrap();
            bcw.write_putc().unwrap();
            bcw.write_putn().unwrap();
            bcw.write_getc().unwrap();
            bcw.write_getn().unwrap();

            let mut bcr = MemReader::new(bcw.unwrap());
            let syntax: Whitespace = Syntax::new();
            syntax.decompile(&mut bcr, &mut writer).unwrap();
        }
        let result = from_utf8(writer.get_ref()).unwrap().replace(" ", "S").replace("\t", "T").replace("\n", "N");
        let expected = vec!(
            "   \t\n", " \n ", " \t  \t \n", " \n\t", " \n\n", " \t\n \t\t\n",
            "\t   ", "\t  \t", "\t  \n", "\t \t ", "\t \t\t",
            "\t\t ", "\t\t\t",
            "\n   \t\n", "\n \t \t\n", "\n \n \t\n", "\n\t  \t\n", "\n\t\t \t\n", "\n\t\n", "\n\n\n",
            "\t\n  ", "\t\n \t", "\t\n\t ", "\t\n\t\t"
            ).concat().replace(" ", "S").replace("\t", "T").replace("\n", "N");
        assert_eq!(result, expected);
    }
}
