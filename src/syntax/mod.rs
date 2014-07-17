//! Compilers and Decompilers.

pub use self::assembly::Assembly;
pub use self::brainfuck::Brainfuck;
pub use self::dt::DT;
pub use self::ook::Ook;
pub use self::whitespace::Whitespace;

use std::io::{BufReader, EndOfFile, InvalidInput, IoResult, standard_error};
use bytecode;
use bytecode::{ByteCodeWriter, ByteCodeReader};

pub type AST = Vec<Instruction>;

#[allow(missing_doc)]
#[deriving(PartialEq, Show, Clone)]
pub enum Instruction {
    WBPush(i64),
    WBDuplicate,
    WBCopy(i64),
    WBSwap,
    WBDiscard,
    WBSlide(i64),
    WBAddition,
    WBSubtraction,
    WBMultiplication,
    WBDivision,
    WBModulo,
    WBStore,
    WBRetrieve,
    WBMark(i64),
    WBCall(i64),
    WBJump(i64),
    WBJumpIfZero(i64),
    WBJumpIfNegative(i64),
    WBReturn,
    WBExit,
    WBPutCharactor,
    WBPutNumber,
    WBGetCharactor,
    WBGetNumber,
}

/// Converter from source code to bytecodes.
pub trait Compiler {
    /// Read source code from buffer, then generate AST.
    fn parse<B: Buffer>(&self, &mut B, &mut AST) -> IoResult<()>;

    /// Read source code from string, then generate AST.
    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        self.parse(&mut BufReader::new(input.as_bytes()), output)
    }

    /// Convert AST to bytecode.
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.parse(input, &mut ast));
        for inst in ast.iter() {
            let ret = match inst {
                &WBPush(n)           => output.write_push(n),
                &WBDuplicate         => output.write_dup(),
                &WBCopy(n)           => output.write_copy(n),
                &WBSwap              => output.write_swap(),
                &WBDiscard           => output.write_discard(),
                &WBSlide(n)          => output.write_slide(n),
                &WBAddition          => output.write_add(),
                &WBSubtraction       => output.write_sub(),
                &WBMultiplication    => output.write_mul(),
                &WBDivision          => output.write_div(),
                &WBModulo            => output.write_mod(),
                &WBStore             => output.write_store(),
                &WBRetrieve          => output.write_retrieve(),
                &WBMark(n)           => output.write_mark(n),
                &WBCall(n)           => output.write_call(n),
                &WBJump(n)           => output.write_jump(n),
                &WBJumpIfZero(n)     => output.write_jumpz(n),
                &WBJumpIfNegative(n) => output.write_jumpn(n),
                &WBReturn            => output.write_return(),
                &WBExit              => output.write_exit(),
                &WBPutCharactor      => output.write_putc(),
                &WBPutNumber         => output.write_putn(),
                &WBGetCharactor      => output.write_getc(),
                &WBGetNumber         => output.write_getn(),
            };
            match ret {
                Err(e) => return Err(e),
                _      => continue,
            }
        }
        Ok(())
    }
}

/// Source code generator from bytecoeds.
pub trait Decompiler {
    /// Generate soruce code from bytecodes.
    fn decompile<R: ByteCodeReader, W: Writer>(&self, &mut R, &mut W) -> IoResult<()>;

    /// Generate AST from byte codes.
    fn disassemble<R: ByteCodeReader>(&self, input: &mut R, output: &mut AST) -> IoResult<()> {
        loop {
            let ret = match input.read_inst() {
                Ok((bytecode::CMD_PUSH, n))     => WBPush(n),
                Ok((bytecode::CMD_DUP, _))      => WBDuplicate,
                Ok((bytecode::CMD_COPY, n))     => WBCopy(n),
                Ok((bytecode::CMD_SWAP, _))     => WBSwap,
                Ok((bytecode::CMD_DISCARD, _))  => WBDiscard,
                Ok((bytecode::CMD_SLIDE, n))    => WBSlide(n),
                Ok((bytecode::CMD_ADD, _))      => WBAddition,
                Ok((bytecode::CMD_SUB, _))      => WBSubtraction,
                Ok((bytecode::CMD_MUL, _))      => WBMultiplication,
                Ok((bytecode::CMD_DIV, _))      => WBDivision,
                Ok((bytecode::CMD_MOD, _))      => WBModulo,
                Ok((bytecode::CMD_STORE, _))    => WBStore,
                Ok((bytecode::CMD_RETRIEVE, _)) => WBRetrieve,
                Ok((bytecode::CMD_MARK, n))     => WBMark(n),
                Ok((bytecode::CMD_CALL, n))     => WBCall(n),
                Ok((bytecode::CMD_JUMP, n))     => WBJump(n),
                Ok((bytecode::CMD_JUMPZ, n))    => WBJumpIfZero(n),
                Ok((bytecode::CMD_JUMPN, n))    => WBJumpIfNegative(n),
                Ok((bytecode::CMD_RETURN, _))   => WBReturn,
                Ok((bytecode::CMD_EXIT, _))     => WBExit,
                Ok((bytecode::CMD_PUTC, _))     => WBPutCharactor,
                Ok((bytecode::CMD_PUTN, _))     => WBPutNumber,
                Ok((bytecode::CMD_GETC, _))     => WBGetCharactor,
                Ok((bytecode::CMD_GETN, _))     => WBGetNumber,
                Err(ref e) if e.kind == EndOfFile => break,
                Err(e)                    => return Err(e),
                _                         => return Err(standard_error(InvalidInput)),
            };
            output.push(ret);
        }
        Ok(())
    }
}

pub mod assembly;
pub mod brainfuck;
pub mod dt;
pub mod ook;
pub mod whitespace;
