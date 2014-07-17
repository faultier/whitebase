//! Parser and Generator for DT.

use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};

use bytecode::ByteCodeReader;
use ir;
use ir::Instruction;
use syntax::{Compiler, Decompiler};
use syntax::whitespace::{Parser, Token, Space, Tab, LF};

static S: &'static str = "ど";
static T: &'static str = "童貞ちゃうわっ！";
static N: &'static str = "…";

struct Scan<'r, T> {
    buffer: &'r mut T
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

struct Tokens<T> {
    iter: T
}

impl<I: Iterator<IoResult<String>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let op = self.iter.next();
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
    fn parse<B: Buffer>(&self, input: &mut B, output: &mut Vec<Instruction>) -> IoResult<()> {
        Parser::new(Tokens { iter: Scan { buffer: input } }).parse(output)
    }
}

impl Decompiler for DT {
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.disassemble(input, &mut ast));
        for inst in ast.iter() {
            let ret = match inst {
                &ir::WBPush(n)              => self.write_num(output, [S, S], n),
                &ir::WBDuplicate            => self.write(output, [S, N, S]),
                &ir::WBCopy(n)              => self.write_num(output, [S, T, S], n),
                &ir::WBSwap                 => self.write(output, [S, N, T]),
                &ir::WBDiscard              => self.write(output, [S, N, N]),
                &ir::WBSlide(n)             => self.write_num(output, [S, T, N], n),
                &ir::WBAddition             => self.write(output, [T, S, S, S]),
                &ir::WBSubtraction          => self.write(output, [T, S, S, T]),
                &ir::WBMultiplication       => self.write(output, [T, S, S, N]),
                &ir::WBDivision             => self.write(output, [T, S, T, S]),
                &ir::WBModulo               => self.write(output, [T, S, T, T]),
                &ir::WBStore                => self.write(output, [T, T, S]),
                &ir::WBRetrieve             => self.write(output, [T, T, T]),
                &ir::WBMark(n)              => self.write_num(output, [N, S, S], n),
                &ir::WBCall(n)              => self.write_num(output, [N, S, T], n),
                &ir::WBJump(n)              => self.write_num(output, [N, S, N], n),
                &ir::WBJumpIfZero(n)        => self.write_num(output, [N, T, S], n),
                &ir::WBJumpIfNegative(n)    => self.write_num(output, [N, T, T], n),
                &ir::WBReturn               => self.write(output, [N, T, N]),
                &ir::WBExit                 => self.write(output, [N, N, N]),
                &ir::WBPutCharactor         => self.write(output, [T, N, S, S]),
                &ir::WBPutNumber            => self.write(output, [T, N, S, T]),
                &ir::WBGetCharactor         => self.write(output, [T, N, T, S]),
                &ir::WBGetNumber            => self.write(output, [T, N, T, T]),
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
    use std::io::{BufReader, MemReader, MemWriter};
    use std::str::from_utf8;

    use super::*;
    use ir::*;
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
        let mut it = super::Scan { buffer: &mut buffer };
        assert_eq!(it.next(), Some(Ok(S.to_string())));
        assert_eq!(it.next(), Some(Ok(T.to_string())));
        assert_eq!(it.next(), Some(Ok(N.to_string())));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_tokenize() {
        let source = vec!(S, "童貞饂飩ちゃうわっ！", T, "\n", N).concat();
        let mut buffer = BufReader::new(source.as_slice().as_bytes());
        let mut it = super::Tokens { iter: super::Scan { buffer: &mut buffer } };
        assert_eq!(it.next(), Some(Ok(Space)));
        assert_eq!(it.next(), Some(Ok(Tab)));
        assert_eq!(it.next(), Some(Ok(LF)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_parse_stack() {
        let source = vec!(
            S, S, S, T, N,      // PUSH 1
            S, N, S,            // DUP
            S, T, S, S, T, N,   // COPY 1
            S, N, T,            // SWAP
            S, N, N,            // DISCARD
            S, T, N, S, T, N,   // SLIDE 1
            ).concat();
        let syntax = DT::new();
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
            T, S, S, S, // ADD
            T, S, S, T, // SUB
            T, S, S, N, // MUL
            T, S, T, S, // DIV
            T, S, T, T, // MOD
            ).concat();
        let syntax = DT::new();
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
            T, T, S,    // STORE
            T, T, T,    // RETRIEVE
            ).concat();
        let syntax = DT::new();
        let mut ast: Vec<Instruction> = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_parse_flow() {
        let source = vec!(
            N, S, S, S, T, N,   // MARK 01
            N, S, T, T, S, N,   // CALL 10
            N, S, N, S, T, N,   // JUMP 01
            N, T, S, T, S, N,   // JUMPZ 10
            N, T, T, S, T, N,   // JUMPN 01
            N, T, N,            // RETURN
            N, N, N,            // EXIT
            ).concat();
        let syntax = DT::new();
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
            T, N, S, S, // PUTC
            T, N, S, T, // PUTN
            T, N, T, S, // GETC
            T, N, T, T, // GETN
            ).concat();
        let syntax = DT::new();
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
