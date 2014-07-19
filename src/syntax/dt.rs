//! Parser and Generator for DT.

#![experimental]

use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};

use bytecode::{ByteCodeReader, ByteCodeWriter};
use ir;
use syntax::{Compiler, Decompiler};
use syntax::whitespace::{Instructions, Token, Space, Tab, LF};

static S: &'static str = "ど";
static T: &'static str = "童貞ちゃうわっ！";
static N: &'static str = "…";

struct Tokens<T> {
    lexemes: T
}

impl<I: Iterator<IoResult<String>>> Tokens<I> {
    pub fn parse(self) -> Instructions<Tokens<I>> { Instructions::new(self) }
}

impl<I: Iterator<IoResult<String>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let op = self.lexemes.next();
        if op.is_none() { return None; }

        let res = op.unwrap();
         match res {
             Err(e) => return Some(Err(e)),
             Ok(_) => (),
        }

        Some(match res.unwrap().as_slice() {
            S => Ok(Space),
            T => Ok(Tab),
            N => Ok(LF),
            _ => Err(standard_error(InvalidInput)),
        })
    }
}

struct Scan<'r, T> {
    buffer: &'r mut T
}

impl<'r, B: Buffer> Scan<'r, B> {
    pub fn tokenize(self) -> Tokens<Scan<'r, B>> { Tokens { lexemes: self } }
}

impl<'r, B: Buffer> Iterator<IoResult<String>> for Scan<'r, B> {
    fn next(&mut self) -> Option<IoResult<String>> {
        'outer: loop {
            match self.buffer.read_char() {
                Ok(c) if c == S.char_at(0) => return Some(Ok(S.to_string())),
                Ok(c) if c == N.char_at(0) => return Some(Ok(N.to_string())),
                Ok(c) if c == T.char_at(0) => {
                    for i in range(1u, 8) {
                        match self.buffer.read_char() {
                            Ok(c) => {
                                if c != T.char_at(i*3) { continue 'outer; }
                            },
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    return Some(Ok(T.to_string()));
                },
                Ok(_) => continue,
                Err(IoError { kind: EndOfFile, ..}) => return None,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

fn scan<'r, B: Buffer>(buffer: &'r mut B) -> Scan<'r, B> { Scan { buffer: buffer } }

/// Compiler and Decompiler for DT.
pub struct DT;

impl DT {
    /// Create a new `DT`.
    pub fn new() -> DT { DT }

    #[inline]
    fn write<W: Writer>(&self, output: &mut W, inst: &[&'static str]) -> IoResult<()> {
        write!(output, "{}", inst.concat())
    }

    #[inline]
    fn write_num<W: Writer>(&self, output: &mut W, cmd: &[&'static str], n: i64) -> IoResult<()> {
        let (flag, value) = if n < 0 { (T, n*-1) } else { (S, n) };
        write!(output, "{}{}{}{}",
               cmd.concat(),
               flag,
               format!("{:t}", value).replace("0", S).replace("1", T),
               N)
    }
}

impl Compiler for DT {
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut it = scan(input).tokenize().parse();
        output.assemble(&mut it)
    }
}

impl Decompiler for DT {
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        for inst in input.disassemble() {
            try!(match inst {
                Ok(ir::StackPush(n))      => self.write_num(output, [S, S], n),
                Ok(ir::StackDuplicate)    => self.write(output, [S, N, S]),
                Ok(ir::StackCopy(n))      => self.write_num(output, [S, T, S], n),
                Ok(ir::StackSwap)         => self.write(output, [S, N, T]),
                Ok(ir::StackDiscard)      => self.write(output, [S, N, N]),
                Ok(ir::StackSlide(n))     => self.write_num(output, [S, T, N], n),
                Ok(ir::Addition)          => self.write(output, [T, S, S, S]),
                Ok(ir::Subtraction)       => self.write(output, [T, S, S, T]),
                Ok(ir::Multiplication)    => self.write(output, [T, S, S, N]),
                Ok(ir::Division)          => self.write(output, [T, S, T, S]),
                Ok(ir::Modulo)            => self.write(output, [T, S, T, T]),
                Ok(ir::HeapStore)         => self.write(output, [T, T, S]),
                Ok(ir::HeapRetrieve)      => self.write(output, [T, T, T]),
                Ok(ir::Mark(n))           => self.write_num(output, [N, S, S], n),
                Ok(ir::Call(n))           => self.write_num(output, [N, S, T], n),
                Ok(ir::Jump(n))           => self.write_num(output, [N, S, N], n),
                Ok(ir::JumpIfZero(n))     => self.write_num(output, [N, T, S], n),
                Ok(ir::JumpIfNegative(n)) => self.write_num(output, [N, T, T], n),
                Ok(ir::Return)            => self.write(output, [N, T, N]),
                Ok(ir::Exit)              => self.write(output, [N, N, N]),
                Ok(ir::PutCharactor)      => self.write(output, [T, N, S, S]),
                Ok(ir::PutNumber)         => self.write(output, [T, N, S, T]),
                Ok(ir::GetCharactor)      => self.write(output, [T, N, T, S]),
                Ok(ir::GetNumber)         => self.write(output, [T, N, T, T]),
                Err(e)                    => Err(e),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, MemReader, MemWriter};
    use std::str::from_utf8;

    use super::*;
    use syntax::*;
    use syntax::whitespace::*;

    use bytecode::ByteCodeWriter;

    static S: &'static str = "ど";
    static T: &'static str = "童貞ちゃうわっ！";
    static N: &'static str = "…";

    #[test]
    fn test_scan() {
        let source = vec!(S, "童貞饂飩ちゃうわっ！", T, "\n", N).concat();
        let mut buffer = BufReader::new(source.as_slice().as_bytes());
        let mut it = super::scan(&mut buffer);
        assert_eq!(it.next(), Some(Ok(S.to_string())));
        assert_eq!(it.next(), Some(Ok(T.to_string())));
        assert_eq!(it.next(), Some(Ok(N.to_string())));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_tokenize() {
        let source = vec!(S, "童貞饂飩ちゃうわっ！", T, "\n", N).concat();
        let mut buffer = BufReader::new(source.as_slice().as_bytes());
        let mut it = super::scan(&mut buffer).tokenize();
        assert_eq!(it.next(), Some(Ok(Space)));
        assert_eq!(it.next(), Some(Ok(Tab)));
        assert_eq!(it.next(), Some(Ok(LF)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_generate() {
        let mut writer = MemWriter::new();
        {
            let mut bcw = MemWriter::new();
            bcw.write_push(-1).unwrap();
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
            let syntax = DT::new();
            syntax.decompile(&mut bcr, &mut writer).unwrap();
        }
        let result = from_utf8(writer.get_ref()).unwrap();
        let expected = vec!(
           "どど童貞ちゃうわっ！童貞ちゃうわっ！…",
           "ど…ど",
           "ど童貞ちゃうわっ！どど童貞ちゃうわっ！ど…",
           "ど…童貞ちゃうわっ！",
           "ど……",
           "ど童貞ちゃうわっ！…ど童貞ちゃうわっ！童貞ちゃうわっ！…",
           "童貞ちゃうわっ！どどど",
           "童貞ちゃうわっ！どど童貞ちゃうわっ！",
           "童貞ちゃうわっ！どど…",
           "童貞ちゃうわっ！ど童貞ちゃうわっ！ど",
           "童貞ちゃうわっ！ど童貞ちゃうわっ！童貞ちゃうわっ！",
           "童貞ちゃうわっ！童貞ちゃうわっ！ど",
           "童貞ちゃうわっ！童貞ちゃうわっ！童貞ちゃうわっ！",
           "…どどど童貞ちゃうわっ！…",
           "…ど童貞ちゃうわっ！ど童貞ちゃうわっ！…",
           "…ど…ど童貞ちゃうわっ！…",
           "…童貞ちゃうわっ！どど童貞ちゃうわっ！…",
           "…童貞ちゃうわっ！童貞ちゃうわっ！ど童貞ちゃうわっ！…",
           "…童貞ちゃうわっ！…",
           "………",
           "童貞ちゃうわっ！…どど",
           "童貞ちゃうわっ！…ど童貞ちゃうわっ！",
           "童貞ちゃうわっ！…童貞ちゃうわっ！ど",
           "童貞ちゃうわっ！…童貞ちゃうわっ！童貞ちゃうわっ！",
        ).concat();
        assert_eq!(result, expected.as_slice());
    }
}
