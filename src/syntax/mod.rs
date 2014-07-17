//! Compilers and Decompilers.

pub use self::assembly::Assembly;
pub use self::brainfuck::Brainfuck;
pub use self::dt::DT;
pub use self::ook::Ook;
pub use self::whitespace::Whitespace;

use std::io::{BufReader, EndOfFile, InvalidInput, IoResult, standard_error};
use bytecode;
use bytecode::{ByteCodeWriter, ByteCodeReader};
use ir;
use ir::Instruction;

/// Converter from source code to bytecodes.
pub trait Compiler {
    /// Read source code from buffer, then generate AST.
    fn parse<B: Buffer>(&self, &mut B, &mut Vec<Instruction>) -> IoResult<()>;

    /// Read source code from string, then generate AST.
    fn parse_str<'a>(&self, input: &'a str, output: &mut Vec<Instruction>) -> IoResult<()> {
        self.parse(&mut BufReader::new(input.as_bytes()), output)
    }

    /// Convert AST to bytecode.
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut ast = vec!();
        try!(self.parse(input, &mut ast));
        for inst in ast.iter() {
            let ret = match inst {
                &ir::WBPush(n)           => output.write_push(n),
                &ir::WBDuplicate         => output.write_dup(),
                &ir::WBCopy(n)           => output.write_copy(n),
                &ir::WBSwap              => output.write_swap(),
                &ir::WBDiscard           => output.write_discard(),
                &ir::WBSlide(n)          => output.write_slide(n),
                &ir::WBAddition          => output.write_add(),
                &ir::WBSubtraction       => output.write_sub(),
                &ir::WBMultiplication    => output.write_mul(),
                &ir::WBDivision          => output.write_div(),
                &ir::WBModulo            => output.write_mod(),
                &ir::WBStore             => output.write_store(),
                &ir::WBRetrieve          => output.write_retrieve(),
                &ir::WBMark(n)           => output.write_mark(n),
                &ir::WBCall(n)           => output.write_call(n),
                &ir::WBJump(n)           => output.write_jump(n),
                &ir::WBJumpIfZero(n)     => output.write_jumpz(n),
                &ir::WBJumpIfNegative(n) => output.write_jumpn(n),
                &ir::WBReturn            => output.write_return(),
                &ir::WBExit              => output.write_exit(),
                &ir::WBPutCharactor      => output.write_putc(),
                &ir::WBPutNumber         => output.write_putn(),
                &ir::WBGetCharactor      => output.write_getc(),
                &ir::WBGetNumber         => output.write_getn(),
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
    fn disassemble<R: ByteCodeReader>(&self, input: &mut R, output: &mut Vec<Instruction>) -> IoResult<()> {
        loop {
            let ret = match input.read_inst() {
                Ok((bytecode::CMD_PUSH, n))     => ir::WBPush(n),
                Ok((bytecode::CMD_DUP, _))      => ir::WBDuplicate,
                Ok((bytecode::CMD_COPY, n))     => ir::WBCopy(n),
                Ok((bytecode::CMD_SWAP, _))     => ir::WBSwap,
                Ok((bytecode::CMD_DISCARD, _))  => ir::WBDiscard,
                Ok((bytecode::CMD_SLIDE, n))    => ir::WBSlide(n),
                Ok((bytecode::CMD_ADD, _))      => ir::WBAddition,
                Ok((bytecode::CMD_SUB, _))      => ir::WBSubtraction,
                Ok((bytecode::CMD_MUL, _))      => ir::WBMultiplication,
                Ok((bytecode::CMD_DIV, _))      => ir::WBDivision,
                Ok((bytecode::CMD_MOD, _))      => ir::WBModulo,
                Ok((bytecode::CMD_STORE, _))    => ir::WBStore,
                Ok((bytecode::CMD_RETRIEVE, _)) => ir::WBRetrieve,
                Ok((bytecode::CMD_MARK, n))     => ir::WBMark(n),
                Ok((bytecode::CMD_CALL, n))     => ir::WBCall(n),
                Ok((bytecode::CMD_JUMP, n))     => ir::WBJump(n),
                Ok((bytecode::CMD_JUMPZ, n))    => ir::WBJumpIfZero(n),
                Ok((bytecode::CMD_JUMPN, n))    => ir::WBJumpIfNegative(n),
                Ok((bytecode::CMD_RETURN, _))   => ir::WBReturn,
                Ok((bytecode::CMD_EXIT, _))     => ir::WBExit,
                Ok((bytecode::CMD_PUTC, _))     => ir::WBPutCharactor,
                Ok((bytecode::CMD_PUTN, _))     => ir::WBPutNumber,
                Ok((bytecode::CMD_GETC, _))     => ir::WBGetCharactor,
                Ok((bytecode::CMD_GETN, _))     => ir::WBGetNumber,
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
