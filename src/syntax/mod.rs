pub use self::assembly::Assembly;
pub use self::brainfuck::Brainfuck;
pub use self::dt::DT;
pub use self::ook::Ook;
pub use self::whitespace::Whitespace;

use std::collections::HashMap;
use std::io::{IoResult, EndOfFile, InvalidInput, standard_error};
use std::iter::Counter;
use bc = bytecode;
use bytecode::{ByteCodeWriter, ByteCodeReader};

pub type AST = Vec<Instruction>;

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

pub trait Syntax {
    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()>;

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()>;

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

    fn disassemble<R: ByteCodeReader>(&self, input: &mut R, output: &mut AST) -> IoResult<()> {
        loop {
            let ret = match input.read_inst() {
                Ok((bc::CMD_PUSH, n))     => WBPush(n),
                Ok((bc::CMD_DUP, _))      => WBDuplicate,
                Ok((bc::CMD_COPY, n))     => WBCopy(n),
                Ok((bc::CMD_SWAP, _))     => WBSwap,
                Ok((bc::CMD_DISCARD, _))  => WBDiscard,
                Ok((bc::CMD_SLIDE, n))    => WBSlide(n),
                Ok((bc::CMD_ADD, _))      => WBAddition,
                Ok((bc::CMD_SUB, _))      => WBSubtraction,
                Ok((bc::CMD_MUL, _))      => WBMultiplication,
                Ok((bc::CMD_DIV, _))      => WBDivision,
                Ok((bc::CMD_MOD, _))      => WBModulo,
                Ok((bc::CMD_STORE, _))    => WBStore,
                Ok((bc::CMD_RETRIEVE, _)) => WBRetrieve,
                Ok((bc::CMD_MARK, n))     => WBMark(n),
                Ok((bc::CMD_CALL, n))     => WBCall(n),
                Ok((bc::CMD_JUMP, n))     => WBJump(n),
                Ok((bc::CMD_JUMPZ, n))    => WBJumpIfZero(n),
                Ok((bc::CMD_JUMPN, n))    => WBJumpIfNegative(n),
                Ok((bc::CMD_RETURN, _))   => WBReturn,
                Ok((bc::CMD_EXIT, _))     => WBExit,
                Ok((bc::CMD_PUTC, _))     => WBPutCharactor,
                Ok((bc::CMD_PUTN, _))     => WBPutNumber,
                Ok((bc::CMD_GETC, _))     => WBGetCharactor,
                Ok((bc::CMD_GETN, _))     => WBGetNumber,
                Err(ref e) if e.kind == EndOfFile => break,
                Err(e)                    => return Err(e),
                _                         => return Err(standard_error(InvalidInput)),
            };
            output.push(ret);
        }
        Ok(())
    }

    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()>;

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

pub mod assembly;
pub mod brainfuck;
pub mod dt;
pub mod ook;
pub mod whitespace;
