use std::io::{InvalidInput, IoResult, standard_error};

use bytecode::ByteCodeReader;
use syntax;
use syntax::{AST, Syntax};
use syntax::whitespace::Whitespace;

macro_rules! dt_write (
    ($w:expr, $inst:expr) => ( write!($w, "{}", ($inst).concat()) )
)

macro_rules! dt_write_num (
    ($w:expr, $cmd:expr, $n:expr) => (
        write!($w, "{}{}{}", ($cmd).concat(),
               (if $n < 0 {
                   format!("{}{:t}", T, $n*-1)
               } else {
                   format!("{}{:t}", S, $n)
               }).replace("0", S).replace("1", T),
               N
        )
    )
)


static S: &'static str = "ど";
static T: &'static str = "童貞ちゃうわっ！";
static N: &'static str = "…";

pub struct DT;

impl Syntax for DT {
    fn new() -> DT { DT }

    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        let mut buffer = String::new();
        for pos in regex!("ど|童貞ちゃうわっ！|…").find_iter(input) {
            let (start, end) = pos;
            match input.slice(start, end) {
                S => buffer.push_char(' '),
                T => buffer.push_char('\t'),
                N => buffer.push_char('\n'),
                _ => return Err(standard_error(InvalidInput)),
            }
        }
        let ws: Whitespace = Syntax::new();
        ws.parse_str(buffer.as_slice(), output)
    }

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
        let source = try!(input.read_to_string());
        self.parse_str(source.as_slice(), output)
    }

    #[allow(unused_variable)]
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.disassemble(input, &mut ast));
        for inst in ast.iter() {
            let ret = match inst {
                &syntax::WBPush(n)              => dt_write_num!(output, [S, S], n),
                &syntax::WBDuplicate            => dt_write!(output, [S, N, S]),
                &syntax::WBCopy(n)              => dt_write_num!(output, [S, T, S], n),
                &syntax::WBSwap                 => dt_write!(output, [S, N, T]),
                &syntax::WBDiscard              => dt_write!(output, [S, N, N]),
                &syntax::WBSlide(n)             => dt_write_num!(output, [S, T, N], n),
                &syntax::WBAddition             => dt_write!(output, [T, S, S, S]),
                &syntax::WBSubtraction          => dt_write!(output, [T, S, S, T]),
                &syntax::WBMultiplication       => dt_write!(output, [T, S, S, N]),
                &syntax::WBDivision             => dt_write!(output, [T, S, T, S]),
                &syntax::WBModulo               => dt_write!(output, [T, S, T, T]),
                &syntax::WBStore                => dt_write!(output, [T, T, S]),
                &syntax::WBRetrieve             => dt_write!(output, [T, T, T]),
                &syntax::WBMark(n)              => dt_write_num!(output, [N, S, S], n),
                &syntax::WBCall(n)              => dt_write_num!(output, [N, S, T], n),
                &syntax::WBJump(n)              => dt_write_num!(output, [N, S, N], n),
                &syntax::WBJumpIfZero(n)        => dt_write_num!(output, [N, T, S], n),
                &syntax::WBJumpIfNegative(n)    => dt_write_num!(output, [N, T, T], n),
                &syntax::WBReturn               => dt_write!(output, [N, T, N]),
                &syntax::WBExit                 => dt_write!(output, [N, N, N]),
                &syntax::WBPutCharactor         => dt_write!(output, [T, N, S, S]),
                &syntax::WBPutNumber            => dt_write!(output, [T, N, S, T]),
                &syntax::WBGetCharactor         => dt_write!(output, [T, N, T, S]),
                &syntax::WBGetNumber            => dt_write!(output, [T, N, T, T]),
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

    static S: &'static str = "ど";
    static T: &'static str = "童貞ちゃうわっ！";
    static N: &'static str = "…";

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
        let syntax: DT = Syntax::new();
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
            T, S, S, S, // ADD
            T, S, S, T, // SUB
            T, S, S, N, // MUL
            T, S, T, S, // DIV
            T, S, T, T, // MOD
            ).concat();
        let syntax: DT = Syntax::new();
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
            T, T, S,    // STORE
            T, T, T,    // RETRIEVE
            ).concat();
        let syntax: DT = Syntax::new();
        let mut ast: AST = vec!();
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
        let syntax: DT = Syntax::new();
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
            T, N, S, S, // PUTC
            T, N, S, T, // PUTN
            T, N, T, S, // GETC
            T, N, T, T, // GETN
            ).concat();
        let syntax: DT = Syntax::new();
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
            let syntax: DT = Syntax::new();
            syntax.decompile(&mut bcr, &mut writer).unwrap();
        }
        let result = from_utf8(writer.get_ref()).unwrap();
        let expected = vec!(
           "どどど童貞ちゃうわっ！…",
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
