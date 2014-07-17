//! Parser and Generator for Whitespace.

use std::collections::HashMap;
use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::iter::{Counter, count};
use std::num::from_str_radix;

use bytecode::ByteCodeReader;
use ir;
use ir::Instruction;
use syntax::{Compiler, Decompiler};

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

#[allow(missing_doc)]
#[deriving(PartialEq, Show)]
pub enum Token {
    Space,
    Tab,
    LF,
}

struct Scan<'r, T> {
    buffer: &'r mut T
}

impl<'r, B: Buffer> Iterator<IoResult<char>> for Scan<'r, B> {
    fn next(&mut self) -> Option<IoResult<char>> {
        loop {
            let ret = match self.buffer.read_char() {
                Ok(' ') => ' ',
                Ok('\t') => '\t',
                Ok('\n') => '\n',
                Ok(_) => continue,
                Err(IoError { kind: EndOfFile, ..}) => return None,
                Err(e) => return Some(Err(e)),
            };
            return Some(Ok(ret));
        }
    }
}

struct Tokens<T> {
    iter: T
}

impl<I: Iterator<IoResult<char>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let c = self.iter.next();
        if c.is_none() { return None; }

        Some(match c.unwrap() {
            Ok(' ')  => Ok(Space),
            Ok('\t') => Ok(Tab),
            Ok('\n') => Ok(LF),
            Ok(_)    => Err(standard_error(InvalidInput)),
            Err(e)   => Err(e),
        })
    }
}

/// Parser for Whitespace.
pub struct Parser<T> {
    iter: T,
    labels: HashMap<String, i64>,
    count: Counter<i64>,
}

fn unknown_instruction(inst: &'static str) -> IoError {
    IoError {
        kind: InvalidInput,
        desc: "syntax error",
        detail: Some(format!("\"{}\" is unknown instruction", inst)),
    }
}

impl<I: Iterator<IoResult<Token>>> Parser<I> {
    /// Create a new `Parser` with token iterator.
    pub fn new(iter: I) -> Parser<I> {
        Parser {
            iter: iter,
            labels: HashMap::new(),
            count: count(1, 1),
        }
    }

    /// Parse Whitespace tokens.
    pub fn parse(&mut self, output: &mut Vec<Instruction>) -> IoResult<()> {
        loop {
            let ret = match self.iter.next() {
                Some(Ok(Space)) => self.parse_stack(),
                Some(Ok(Tab)) => match self.iter.next() {
                    Some(Ok(Space)) => self.parse_arithmetic(),
                    Some(Ok(Tab)) => self.parse_heap(),
                    Some(Ok(LF)) => self.parse_io(),
                    _ => Err(standard_error(InvalidInput)),
                },
                Some(Ok(LF)) => self.parse_flow(),
                Some(Err(e)) => Err(e),
                None => break,
            };
            match ret {
                Ok(inst) => output.push(inst),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn parse_value(&mut self) -> IoResult<String> {
        let mut value = String::new();
        loop {
            match self.iter.next() {
                Some(Ok(Space)) => value.push_char('0'),
                Some(Ok(Tab)) => value.push_char('1'),
                Some(Ok(LF)) => break,
                Some(Err(e)) => return Err(e),
                None => return Err(IoError {
                    kind: InvalidInput,
                    desc: "syntax error",
                    detail: Some("no value terminator".to_string()),
                }),
            }
        }
        Ok(value)
    }

    fn parse_sign(&mut self) -> IoResult<bool> {
        match self.iter.next() {
            Some(Ok(Space)) => Ok(true),
            Some(Ok(Tab)) => Ok(false),
            Some(Ok(LF)) | None => Err(IoError {
                kind: InvalidInput,
                desc: "invalid value format",
                detail: Some("no sign".to_string()),
            }),
            Some(Err(e)) => Err(e),
        }
    }

    fn parse_number(&mut self) -> IoResult<i64> {
        let positive = try!(self.parse_sign());
        let value = try!(self.parse_value());
        match from_str_radix::<i64>(value.as_slice(), 2) {
            Some(n) => Ok(if positive { n } else { n * -1 }),
            None => Err(standard_error(InvalidInput)),
        }
    }

    fn parse_label(&mut self) -> IoResult<i64> {
        let label = try!(self.parse_value());
        match self.labels.find_copy(&label) {
            Some(val) => Ok(val),
            None => {
                let val = self.count.next().unwrap();
                self.labels.insert(label, val);
                Ok(val)
            },
        }
    }

    fn parse_stack(&mut self) -> IoResult<Instruction> {
        match self.iter.next() {
            Some(Ok(Space)) => Ok(ir::WBPush(try!(self.parse_number()))),
            Some(Ok(LF)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBDuplicate),
                Some(Ok(Tab)) => Ok(ir::WBSwap),
                Some(Ok(LF)) => Ok(ir::WBDiscard),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("SN")),
            },
            Some(Ok(Tab)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBCopy(try!(self.parse_number()))),
                Some(Ok(LF)) => Ok(ir::WBSlide(try!(self.parse_number()))),
                Some(Ok(Tab)) => Err(unknown_instruction("STT")),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("ST")),
            },
            Some(Err(e)) => Err(e),
            None => Err(unknown_instruction("S")),
        }
    }

    fn parse_arithmetic(&mut self) -> IoResult<Instruction> {
        match self.iter.next() {
            Some(Ok(Space)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBAddition),
                Some(Ok(Tab)) => Ok(ir::WBSubtraction),
                Some(Ok(LF)) => Ok(ir::WBMultiplication),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("TSS")),
            },
            Some(Ok(Tab)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBDivision),
                Some(Ok(Tab)) => Ok(ir::WBModulo),
                Some(Ok(LF)) => Err(unknown_instruction("TSTN")),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("TST")),
            },
            Some(Ok(LF)) => Err(unknown_instruction("TSN")),
            Some(Err(e)) => Err(e),
            None => Err(unknown_instruction("TS")),
        }
    }

    fn parse_heap(&mut self) -> IoResult<Instruction> {
        match self.iter.next() {
            Some(Ok(Space)) => Ok(ir::WBStore),
            Some(Ok(Tab)) => Ok(ir::WBRetrieve),
            Some(Err(e)) => Err(e),
            Some(Ok(LF)) => Err(unknown_instruction("TTN")),
            None => Err(unknown_instruction("TT")),
        }
    }

    fn parse_flow(&mut self) -> IoResult<Instruction> {
        match self.iter.next() {
            Some(Ok(Space)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBMark(try!(self.parse_label()))),
                Some(Ok(Tab)) => Ok(ir::WBCall(try!(self.parse_label()))),
                Some(Ok(LF)) => Ok(ir::WBJump(try!(self.parse_label()))),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("NS")),
            },
            Some(Ok(Tab)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBJumpIfZero(try!(self.parse_label()))),
                Some(Ok(Tab)) => Ok(ir::WBJumpIfNegative(try!(self.parse_label()))),
                Some(Ok(LF)) => Ok(ir::WBReturn),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("NT")),
            },
            Some(Ok(LF)) => match self.iter.next() {
                Some(Ok(LF)) => Ok(ir::WBExit),
                Some(Ok(Space)) => Err(unknown_instruction("NNS")),
                Some(Ok(Tab)) => Err(unknown_instruction("NNT")),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("NN")),
            },
            Some(Err(e)) => Err(e),
            None => Err(unknown_instruction("N")),
        }
    }

    fn parse_io(&mut self) -> IoResult<Instruction> {
        match self.iter.next() {
            Some(Ok(Space)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBPutCharactor),
                Some(Ok(Tab)) => Ok(ir::WBPutNumber),
                Some(Ok(LF)) => Err(unknown_instruction("TNSN")),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("TNS")),
            },
            Some(Ok(Tab)) => match self.iter.next() {
                Some(Ok(Space)) => Ok(ir::WBGetCharactor),
                Some(Ok(Tab)) => Ok(ir::WBGetNumber),
                Some(Ok(LF)) => Err(unknown_instruction("TNTN")),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("TNT")),
            },
            Some(Ok(LF)) => Err(unknown_instruction("TNN")),
            Some(Err(e)) => Err(e),
            None => Err(unknown_instruction("TN")),
        }
    }
}

/// Compiler and Decompiler for Whitespace.
pub struct Whitespace;

impl Whitespace {
    /// Create a new `Whitespace`.
    pub fn new() -> Whitespace { Whitespace }
}

impl Compiler for Whitespace {
    fn parse<B: Buffer>(&self, input: &mut B, output: &mut Vec<Instruction>) -> IoResult<()> {
        Parser::new(Tokens { iter: Scan { buffer: input } }).parse(output)
    }
}

impl Decompiler for Whitespace {
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.disassemble(input, &mut ast));
        for inst in ast.iter() {
            let ret = match inst {
                &ir::WBPush(n)              => write_num!(output, "  ", n),
                &ir::WBDuplicate            => write!(output, " \n "),
                &ir::WBCopy(n)              => write_num!(output, " \t ", n),
                &ir::WBSwap                 => write!(output, " \n\t"),
                &ir::WBDiscard              => write!(output, " \n\n"),
                &ir::WBSlide(n)             => write_num!(output, " \t\n", n),
                &ir::WBAddition             => write!(output, "\t   "),
                &ir::WBSubtraction          => write!(output, "\t  \t"),
                &ir::WBMultiplication       => write!(output, "\t  \n"),
                &ir::WBDivision             => write!(output, "\t \t "),
                &ir::WBModulo               => write!(output, "\t \t\t"),
                &ir::WBStore                => write!(output, "\t\t "),
                &ir::WBRetrieve             => write!(output, "\t\t\t"),
                &ir::WBMark(n)              => write_num!(output, "\n  ", n),
                &ir::WBCall(n)              => write_num!(output, "\n \t", n),
                &ir::WBJump(n)              => write_num!(output, "\n \n", n),
                &ir::WBJumpIfZero(n)        => write_num!(output, "\n\t ", n),
                &ir::WBJumpIfNegative(n)    => write_num!(output, "\n\t\t", n),
                &ir::WBReturn               => write!(output, "\n\t\n"),
                &ir::WBExit                 => write!(output, "\n\n\n"),
                &ir::WBPutCharactor         => write!(output, "\t\n  "),
                &ir::WBPutNumber            => write!(output, "\t\n \t"),
                &ir::WBGetCharactor         => write!(output, "\t\n\t "),
                &ir::WBGetNumber            => write!(output, "\t\n\t\t"),
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
    use ir::*;
    use syntax::*;

    use std::io::BufReader;

    #[test]
    fn test_scan() {
        let mut buffer = BufReader::new(" [\t饂飩]\n".as_bytes());
        let mut it = super::Scan { buffer: &mut buffer };
        assert_eq!(it.next(), Some(Ok(' ')));
        assert_eq!(it.next(), Some(Ok('\t')));
        assert_eq!(it.next(), Some(Ok('\n')));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_tokenize() {
        let mut buffer = BufReader::new(" [\t饂飩]\n".as_bytes());
        let mut it = super::Tokens { iter: super::Scan { buffer: &mut buffer } };
        assert_eq!(it.next(), Some(Ok(Space)));
        assert_eq!(it.next(), Some(Ok(Tab)));
        assert_eq!(it.next(), Some(Ok(LF)));
        assert!(it.next().is_none());
    }

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
        let syntax = Whitespace::new();
        let mut ast: Vec<Instruction> = vec!();
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
        let syntax = Whitespace::new();
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
            "\t\t ",    // STORE
            "\t\t\t",   // RETRIEVE
            ).concat();
        let syntax = Whitespace::new();
        let mut ast: Vec<Instruction> = vec!();
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
        let syntax = Whitespace::new();
        let mut ast: Vec<Instruction> = vec!();
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
        let syntax = Whitespace::new();
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
            let syntax = Whitespace::new();
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
