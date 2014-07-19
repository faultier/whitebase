//! Parser and Generator for Whitespace.

#![experimental]

use std::collections::HashMap;
use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::iter::{Counter, count};
use std::num::from_str_radix;

use bytecode::{ByteCodeReader, ByteCodeWriter};
use ir;
use ir::Instruction;
use syntax::{Compile, Decompile};

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

fn unknown_instruction(inst: &'static str) -> IoError {
    IoError {
        kind: InvalidInput,
        desc: "syntax error",
        detail: Some(format!("\"{}\" is unknown instruction", inst)),
    }
}

/// An iterator that convert to IR from whitespace tokens on each iteration.
pub struct Instructions<T> {
    tokens: T,
    labels: HashMap<String, i64>,
    count: Counter<i64>,
}

impl<I: Iterator<IoResult<Token>>> Instructions<I> {
    /// Create an iterator that convert to IR from tokens on each iteration.
    pub fn new(iter: I) -> Instructions<I> {
        Instructions {
            tokens: iter,
            labels: HashMap::new(),
            count: count(1, 1),
        }
    }

    fn parse_value(&mut self) -> IoResult<String> {
        let mut value = String::new();
        loop {
            match self.tokens.next() {
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
        match self.tokens.next() {
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
        match self.tokens.next() {
            Some(Ok(Space)) => Ok(ir::WBPush(try!(self.parse_number()))),
            Some(Ok(LF)) => match self.tokens.next() {
                Some(Ok(Space)) => Ok(ir::WBDuplicate),
                Some(Ok(Tab)) => Ok(ir::WBSwap),
                Some(Ok(LF)) => Ok(ir::WBDiscard),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("SN")),
            },
            Some(Ok(Tab)) => match self.tokens.next() {
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
        match self.tokens.next() {
            Some(Ok(Space)) => match self.tokens.next() {
                Some(Ok(Space)) => Ok(ir::WBAddition),
                Some(Ok(Tab)) => Ok(ir::WBSubtraction),
                Some(Ok(LF)) => Ok(ir::WBMultiplication),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("TSS")),
            },
            Some(Ok(Tab)) => match self.tokens.next() {
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
        match self.tokens.next() {
            Some(Ok(Space)) => Ok(ir::WBStore),
            Some(Ok(Tab)) => Ok(ir::WBRetrieve),
            Some(Err(e)) => Err(e),
            Some(Ok(LF)) => Err(unknown_instruction("TTN")),
            None => Err(unknown_instruction("TT")),
        }
    }

    fn parse_flow(&mut self) -> IoResult<Instruction> {
        match self.tokens.next() {
            Some(Ok(Space)) => match self.tokens.next() {
                Some(Ok(Space)) => Ok(ir::WBMark(try!(self.parse_label()))),
                Some(Ok(Tab)) => Ok(ir::WBCall(try!(self.parse_label()))),
                Some(Ok(LF)) => Ok(ir::WBJump(try!(self.parse_label()))),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("NS")),
            },
            Some(Ok(Tab)) => match self.tokens.next() {
                Some(Ok(Space)) => Ok(ir::WBJumpIfZero(try!(self.parse_label()))),
                Some(Ok(Tab)) => Ok(ir::WBJumpIfNegative(try!(self.parse_label()))),
                Some(Ok(LF)) => Ok(ir::WBReturn),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("NT")),
            },
            Some(Ok(LF)) => match self.tokens.next() {
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
        match self.tokens.next() {
            Some(Ok(Space)) => match self.tokens.next() {
                Some(Ok(Space)) => Ok(ir::WBPutCharactor),
                Some(Ok(Tab)) => Ok(ir::WBPutNumber),
                Some(Ok(LF)) => Err(unknown_instruction("TNSN")),
                Some(Err(e)) => Err(e),
                None => Err(unknown_instruction("TNS")),
            },
            Some(Ok(Tab)) => match self.tokens.next() {
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

impl<I: Iterator<IoResult<Token>>> Iterator<IoResult<Instruction>> for Instructions<I> {
    fn next(&mut self) -> Option<IoResult<Instruction>> {
        match self.tokens.next() {
            Some(Ok(Space)) => Some(self.parse_stack()),
            Some(Ok(Tab)) => match self.tokens.next() {
                Some(Ok(Space)) => Some(self.parse_arithmetic()),
                Some(Ok(Tab))   => Some(self.parse_heap()),
                Some(Ok(LF))    => Some(self.parse_io()),
                _               => Some(Err(standard_error(InvalidInput))),
            },
            Some(Ok(LF)) => Some(self.parse_flow()),
            Some(Err(e)) => Some(Err(e)),
            None         => None,
        }
    }
}

#[allow(missing_doc)]
#[deriving(PartialEq, Show)]
pub enum Token {
    Space,
    Tab,
    LF,
}

struct Tokens<T> {
    lexemes: T
}

impl<I: Iterator<IoResult<char>>> Tokens<I> {
    pub fn parse(self) -> Instructions<Tokens<I>> { Instructions::new(self) }
}

impl<I: Iterator<IoResult<char>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let c = self.lexemes.next();
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

struct Scan<'r, T> {
    buffer: &'r mut T
}

impl<'r, B: Buffer> Scan<'r, B> {
    pub fn tokenize(self) -> Tokens<Scan<'r, B>> { Tokens { lexemes: self } }
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

fn scan<'r, B: Buffer>(buffer: &'r mut B) -> Scan<'r, B> { Scan { buffer: buffer } }

/// Compiler and Decompiler for Whitespace.
pub struct Whitespace;

impl Whitespace {
    /// Create a new `Whitespace`.
    pub fn new() -> Whitespace { Whitespace }
}

impl Compile for Whitespace {
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut it = scan(input).tokenize().parse();
        output.assemble(&mut it)
    }
}

impl Decompile for Whitespace {
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        for inst in input.disassemble() {
            try!(match inst {
                Ok(ir::WBPush(n))            => write_num!(output, "  ", n),
                Ok(ir::WBDuplicate)          => write!(output, " \n "),
                Ok(ir::WBCopy(n))            => write_num!(output, " \t ", n),
                Ok(ir::WBSwap)               => write!(output, " \n\t"),
                Ok(ir::WBDiscard)            => write!(output, " \n\n"),
                Ok(ir::WBSlide(n))           => write_num!(output, " \t\n", n),
                Ok(ir::WBAddition)           => write!(output, "\t   "),
                Ok(ir::WBSubtraction)        => write!(output, "\t  \t"),
                Ok(ir::WBMultiplication)     => write!(output, "\t  \n"),
                Ok(ir::WBDivision)           => write!(output, "\t \t "),
                Ok(ir::WBModulo)             => write!(output, "\t \t\t"),
                Ok(ir::WBStore)              => write!(output, "\t\t "),
                Ok(ir::WBRetrieve)           => write!(output, "\t\t\t"),
                Ok(ir::WBMark(n))            => write_num!(output, "\n  ", n),
                Ok(ir::WBCall(n))            => write_num!(output, "\n \t", n),
                Ok(ir::WBJump(n))            => write_num!(output, "\n \n", n),
                Ok(ir::WBJumpIfZero(n))      => write_num!(output, "\n\t ", n),
                Ok(ir::WBJumpIfNegative(n))  => write_num!(output, "\n\t\t", n),
                Ok(ir::WBReturn)             => write!(output, "\n\t\n"),
                Ok(ir::WBExit)               => write!(output, "\n\n\n"),
                Ok(ir::WBPutCharactor)       => write!(output, "\t\n  "),
                Ok(ir::WBPutNumber)          => write!(output, "\t\n \t"),
                Ok(ir::WBGetCharactor)       => write!(output, "\t\n\t "),
                Ok(ir::WBGetNumber)          => write!(output, "\t\n\t\t"),
                Err(e)                       => Err(e),
            })
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
        let mut it = super::scan(&mut buffer);
        assert_eq!(it.next(), Some(Ok(' ')));
        assert_eq!(it.next(), Some(Ok('\t')));
        assert_eq!(it.next(), Some(Ok('\n')));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_tokenize() {
        let mut buffer = BufReader::new(" [\t饂飩]\n".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize();
        assert_eq!(it.next(), Some(Ok(Space)));
        assert_eq!(it.next(), Some(Ok(Tab)));
        assert_eq!(it.next(), Some(Ok(LF)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_parse() {
        let source = vec!(
            "   \t\n",      // PUSH 1
            " \n ",         // DUP
            " \t  \t\n",    // COPY 1
            " \n\t",        // SWAP
            " \n\n",        // DISCARD
            " \t\n \t\n",   // SLIDE 1
            "\t   ",        // ADD
            "\t  \t",       // SUB
            "\t  \n",       // MUL
            "\t \t ",       // DIV
            "\t \t\t",      // MOD
            "\t\t ",        // STORE
            "\t\t\t",       // RETRIEVE
            "\n   \t\n",    // MARK 01
            "\n \t\t \n",   // CALL 10
            "\n \n \t\n",   // JUMP 01
            "\n\t \t \n",   // JUMPZ 10
            "\n\t\t \t\n",  // JUMPN 01
            "\n\t\n",       // RETURN
            "\n\n\n",       // EXIT
            "\t\n  ",       // PUTC
            "\t\n \t",      // PUTN
            "\t\n\t ",      // GETC
            "\t\n\t\t",     // GETN
            ).concat();
        let mut buffer = BufReader::new(source.as_slice().as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(1))));
        assert_eq!(it.next(), Some(Ok(WBDuplicate)));
        assert_eq!(it.next(), Some(Ok(WBCopy(1))));
        assert_eq!(it.next(), Some(Ok(WBSwap)));
        assert_eq!(it.next(), Some(Ok(WBDiscard)));
        assert_eq!(it.next(), Some(Ok(WBSlide(1))));
        assert_eq!(it.next(), Some(Ok(WBAddition)));
        assert_eq!(it.next(), Some(Ok(WBSubtraction)));
        assert_eq!(it.next(), Some(Ok(WBMultiplication)));
        assert_eq!(it.next(), Some(Ok(WBDivision)));
        assert_eq!(it.next(), Some(Ok(WBModulo)));
        assert_eq!(it.next(), Some(Ok(WBStore)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBMark(1))));
        assert_eq!(it.next(), Some(Ok(WBCall(2))));
        assert_eq!(it.next(), Some(Ok(WBJump(1))));
        assert_eq!(it.next(), Some(Ok(WBJumpIfZero(2))));
        assert_eq!(it.next(), Some(Ok(WBJumpIfNegative(1))));
        assert_eq!(it.next(), Some(Ok(WBReturn)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBPutCharactor)));
        assert_eq!(it.next(), Some(Ok(WBPutNumber)));
        assert_eq!(it.next(), Some(Ok(WBGetCharactor)));
        assert_eq!(it.next(), Some(Ok(WBGetNumber)));
        assert!(it.next().is_none());
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
