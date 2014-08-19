//! Bytecode utilities.

#![unstable]

use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};

use ir;
use ir::Instruction;

pub static IMP_STACK: u8      = 0b0011 << 4;
pub static IMP_ARITHMETIC: u8 = 0b1000 << 4;
pub static IMP_HEAP: u8       = 0b1010 << 4;
pub static IMP_FLOW: u8       = 0b0111 << 4;
pub static IMP_IO: u8         = 0b1001 << 4;

pub static CMD_PUSH: u8     = IMP_STACK + 0b0011;
pub static CMD_DUP: u8      = IMP_STACK + 0b0100;
pub static CMD_COPY: u8     = IMP_STACK + 0b1000;
pub static CMD_SWAP: u8     = IMP_STACK + 0b0110;
pub static CMD_DISCARD: u8  = IMP_STACK + 0b0101;
pub static CMD_SLIDE: u8    = IMP_STACK + 0b1001;
pub static CMD_ADD: u8      = IMP_ARITHMETIC + 0b0000;
pub static CMD_SUB: u8      = IMP_ARITHMETIC + 0b0010;
pub static CMD_MUL: u8      = IMP_ARITHMETIC + 0b0001;
pub static CMD_DIV: u8      = IMP_ARITHMETIC + 0b1000;
pub static CMD_MOD: u8      = IMP_ARITHMETIC + 0b1010;
pub static CMD_STORE: u8    = IMP_HEAP + 0b0011;
pub static CMD_RETRIEVE: u8 = IMP_HEAP + 0b1011;
pub static CMD_MARK: u8     = IMP_FLOW + 0b0000;
pub static CMD_CALL: u8     = IMP_FLOW + 0b0010;
pub static CMD_JUMP: u8     = IMP_FLOW + 0b0001;
pub static CMD_JUMPZ: u8    = IMP_FLOW + 0b1000;
pub static CMD_JUMPN: u8    = IMP_FLOW + 0b1010;
pub static CMD_RETURN: u8   = IMP_FLOW + 0b1001;
pub static CMD_EXIT: u8     = IMP_FLOW + 0b0101;
pub static CMD_PUTC: u8     = IMP_IO + 0b0000;
pub static CMD_PUTN: u8     = IMP_IO + 0b0010;
pub static CMD_GETC: u8     = IMP_IO + 0b1000;
pub static CMD_GETN: u8     = IMP_IO + 0b1010;

#[experimental]
/// Bytecodes writer.
pub trait ByteCodeWriter {
    /// Compile a instruction to bytecodes.
    fn assemble<I: Iterator<IoResult<Instruction>>>(&mut self, &mut I) -> IoResult<()>;
    /// Writes a push instruction.
    fn write_push(&mut self, n: i64) -> IoResult<()>;
    /// Writes a duplicate instruction.
    fn write_dup(&mut self) -> IoResult<()>;
    /// Writes a copy instruction.
    fn write_copy(&mut self, n: i64) -> IoResult<()>;
    /// Writes a swap instruction.
    fn write_swap(&mut self) -> IoResult<()>;
    /// Writes a discard instruction.
    fn write_discard(&mut self) -> IoResult<()>;
    /// Writes a slide instruction.
    fn write_slide(&mut self, n: i64) -> IoResult<()>;
    /// Writes a addition instruction.
    fn write_add(&mut self) -> IoResult<()>;
    /// Writes a subtraction instruction.
    fn write_sub(&mut self) -> IoResult<()>;
    /// Writes a multiplication instruction.
    fn write_mul(&mut self) -> IoResult<()>;
    /// Writes a division instruction.
    fn write_div(&mut self) -> IoResult<()>;
    /// Writes a modulo instruction.
    fn write_mod(&mut self) -> IoResult<()>;
    /// Writes a store instruction.
    fn write_store(&mut self) -> IoResult<()>;
    /// Writes a retrieve instruction.
    fn write_retrieve(&mut self) -> IoResult<()>;
    /// Writes a mark instruction.
    fn write_mark(&mut self, n: i64) -> IoResult<()>;
    /// Writes a call instruction.
    fn write_call(&mut self, n: i64) -> IoResult<()>;
    /// Writes a jump instruction.
    fn write_jump(&mut self, n: i64) -> IoResult<()>;
    /// Writes a conditional jump instruction.
    fn write_jumpz(&mut self, n: i64) -> IoResult<()>;
    /// Writes a conditional jump instruction.
    fn write_jumpn(&mut self, n: i64) -> IoResult<()>;
    /// Writes a return instruction.
    fn write_return(&mut self) -> IoResult<()>;
    /// Writes a exit instruction.
    fn write_exit(&mut self) -> IoResult<()>;
    /// Writes a character put instruction.
    fn write_putc(&mut self) -> IoResult<()>;
    /// Writes a number put instruction.
    fn write_putn(&mut self) -> IoResult<()>;
    /// Writes a character get instruction.
    fn write_getc(&mut self) -> IoResult<()>;
    /// Writes a number get instruction.
    fn write_getn(&mut self) -> IoResult<()>;
}

impl<W: Writer> ByteCodeWriter for W {
    fn assemble<I: Iterator<IoResult<Instruction>>>(&mut self, iter: &mut I) -> IoResult<()> {
        for inst in *iter {
            try!(match inst {
                Ok(ir::StackPush(n))      => self.write_push(n),
                Ok(ir::StackDuplicate)    => self.write_dup(),
                Ok(ir::StackCopy(n))      => self.write_copy(n),
                Ok(ir::StackSwap)         => self.write_swap(),
                Ok(ir::StackDiscard)      => self.write_discard(),
                Ok(ir::StackSlide(n))     => self.write_slide(n),
                Ok(ir::Addition)          => self.write_add(),
                Ok(ir::Subtraction)       => self.write_sub(),
                Ok(ir::Multiplication)    => self.write_mul(),
                Ok(ir::Division)          => self.write_div(),
                Ok(ir::Modulo)            => self.write_mod(),
                Ok(ir::HeapStore)         => self.write_store(),
                Ok(ir::HeapRetrieve)      => self.write_retrieve(),
                Ok(ir::Mark(n))           => self.write_mark(n),
                Ok(ir::Call(n))           => self.write_call(n),
                Ok(ir::Jump(n))           => self.write_jump(n),
                Ok(ir::JumpIfZero(n))     => self.write_jumpz(n),
                Ok(ir::JumpIfNegative(n)) => self.write_jumpn(n),
                Ok(ir::Return)            => self.write_return(),
                Ok(ir::Exit)              => self.write_exit(),
                Ok(ir::PutCharactor)      => self.write_putc(),
                Ok(ir::PutNumber)         => self.write_putn(),
                Ok(ir::GetCharactor)      => self.write_getc(),
                Ok(ir::GetNumber)         => self.write_getn(),
                Err(e)                      => Err(e),
            });
        }
        Ok(())
    }

    fn write_push(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_PUSH));
        self.write_be_i64(n)
    }

    fn write_dup(&mut self) -> IoResult<()> {
        self.write_u8(CMD_DUP)
    }

    fn write_copy(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_COPY));
        self.write_be_i64(n)
    }

    fn write_swap(&mut self) -> IoResult<()> {
        self.write_u8(CMD_SWAP)
    }

    fn write_discard(&mut self) -> IoResult<()> {
        self.write_u8(CMD_DISCARD)
    }

    fn write_slide(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_SLIDE));
        self.write_be_i64(n)
    }

    fn write_add(&mut self) -> IoResult<()> {
        self.write_u8(CMD_ADD)
    }

    fn write_sub(&mut self) -> IoResult<()> {
        self.write_u8(CMD_SUB)
    }

    fn write_mul(&mut self) -> IoResult<()> {
        self.write_u8(CMD_MUL)
    }

    fn write_div(&mut self) -> IoResult<()> {
        self.write_u8(CMD_DIV)
    }

    fn write_mod(&mut self) -> IoResult<()> {
        self.write_u8(CMD_MOD)
    }

    fn write_store(&mut self) -> IoResult<()> {
        self.write_u8(CMD_STORE)
    }

    fn write_retrieve(&mut self) -> IoResult<()> {
        self.write_u8(CMD_RETRIEVE)
    }

    fn write_mark(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_MARK));
        self.write_be_i64(n)
    }

    fn write_call(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_CALL));
        self.write_be_i64(n)
    }

    fn write_jump(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_JUMP));
        self.write_be_i64(n)
    }

    fn write_jumpz(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_JUMPZ));
        self.write_be_i64(n)
    }

    fn write_jumpn(&mut self, n: i64) -> IoResult<()> {
        try!(self.write_u8(CMD_JUMPN));
        self.write_be_i64(n)
    }

    fn write_return(&mut self) -> IoResult<()> {
        self.write_u8(CMD_RETURN)
    }

    fn write_exit(&mut self) -> IoResult<()> {
        self.write_u8(CMD_EXIT)
    }

    fn write_putn(&mut self) -> IoResult<()> {
        self.write_u8(CMD_PUTN)
    }

    fn write_putc(&mut self) -> IoResult<()> {
        self.write_u8(CMD_PUTC)
    }

    fn write_getc(&mut self) -> IoResult<()> {
        self.write_u8(CMD_GETC)
    }

    fn write_getn(&mut self) -> IoResult<()> {
        self.write_u8(CMD_GETN)
    }
}

#[experimental]
/// An iterator that convert to IR from bytes on each iteration, `read_inst()` encounters `EndOfFile`.
pub struct Instructions<'r, T> {
    reader: &'r mut T
}

impl<'r, B: ByteCodeReader> Iterator<IoResult<Instruction>> for Instructions<'r, B> {
    fn next(&mut self) -> Option<IoResult<Instruction>> {
        match self.reader.read_inst() {
            Ok((CMD_PUSH, n))     => Some(Ok(ir::StackPush(n))),
            Ok((CMD_DUP, _))      => Some(Ok(ir::StackDuplicate)),
            Ok((CMD_COPY, n))     => Some(Ok(ir::StackCopy(n))),
            Ok((CMD_SWAP, _))     => Some(Ok(ir::StackSwap)),
            Ok((CMD_DISCARD, _))  => Some(Ok(ir::StackDiscard)),
            Ok((CMD_SLIDE, n))    => Some(Ok(ir::StackSlide(n))),
            Ok((CMD_ADD, _))      => Some(Ok(ir::Addition)),
            Ok((CMD_SUB, _))      => Some(Ok(ir::Subtraction)),
            Ok((CMD_MUL, _))      => Some(Ok(ir::Multiplication)),
            Ok((CMD_DIV, _))      => Some(Ok(ir::Division)),
            Ok((CMD_MOD, _))      => Some(Ok(ir::Modulo)),
            Ok((CMD_STORE, _))    => Some(Ok(ir::HeapStore)),
            Ok((CMD_RETRIEVE, _)) => Some(Ok(ir::HeapRetrieve)),
            Ok((CMD_MARK, n))     => Some(Ok(ir::Mark(n))),
            Ok((CMD_CALL, n))     => Some(Ok(ir::Call(n))),
            Ok((CMD_JUMP, n))     => Some(Ok(ir::Jump(n))),
            Ok((CMD_JUMPZ, n))    => Some(Ok(ir::JumpIfZero(n))),
            Ok((CMD_JUMPN, n))    => Some(Ok(ir::JumpIfNegative(n))),
            Ok((CMD_RETURN, _))   => Some(Ok(ir::Return)),
            Ok((CMD_EXIT, _))     => Some(Ok(ir::Exit)),
            Ok((CMD_PUTC, _))     => Some(Ok(ir::PutCharactor)),
            Ok((CMD_PUTN, _))     => Some(Ok(ir::PutNumber)),
            Ok((CMD_GETC, _))     => Some(Ok(ir::GetCharactor)),
            Ok((CMD_GETN, _))     => Some(Ok(ir::GetNumber)),
            Err(IoError { kind: EndOfFile, ..}) => None,
            Err(e) => Some(Err(e)),
            _ => Some(Err(standard_error(InvalidInput))),
        }
    }
}

#[experimental]
/// Bytecodes reader.
pub trait ByteCodeReader: Reader + Seek {
    /// Read the next instruction bytes from the underlying stream.
    ///
    /// # Error
    ///
    /// If an I/O error occurs, or EOF, then this function will return `Err`.
    fn read_inst(&mut self) -> IoResult<(u8, i64)>;

    /// Create an iterator that convert to IR from bytes on each iteration
    /// until EOF.
    ///
    /// # Error
    ///
    /// Any error other than `EndOfFile` that is produced by the underlying Reader
    /// is returned by the iterator and should be handled by the caller.
    fn disassemble<'r>(&'r mut self) -> Instructions<'r, Self> {
        Instructions { reader: self }
    }
}

impl<R: Reader + Seek> ByteCodeReader for R {
    fn read_inst(&mut self) -> IoResult<(u8, i64)> {
        match self.read_u8() {
            Ok(n) if n == CMD_PUSH || n == CMD_COPY || n == CMD_SLIDE || n == CMD_MARK || n == CMD_CALL || n == CMD_JUMP || n == CMD_JUMPZ || n == CMD_JUMPN => {
                Ok((n, try!(self.read_be_i64())))
            },
            Ok(n) => Ok((n, 0)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{IoResult, MemReader, MemWriter};
    use ir;
    use super::{ByteCodeReader, ByteCodeWriter};

    #[test]
    fn test_readwrite() {
        let mut writer = MemWriter::new();
        writer.write_push(-1).unwrap();
        writer.write_dup().unwrap();
        writer.write_copy(1).unwrap();
        writer.write_swap().unwrap();
        writer.write_discard().unwrap();
        writer.write_slide(2).unwrap();
        writer.write_add().unwrap();
        writer.write_sub().unwrap();
        writer.write_mul().unwrap();
        writer.write_div().unwrap();
        writer.write_mod().unwrap();
        writer.write_store().unwrap();
        writer.write_retrieve().unwrap();
        writer.write_mark(-1).unwrap();
        writer.write_call(1).unwrap();
        writer.write_jump(-1).unwrap();
        writer.write_jumpz(1).unwrap();
        writer.write_jumpn(-1).unwrap();
        writer.write_return().unwrap();
        writer.write_exit().unwrap();
        writer.write_putc().unwrap();
        writer.write_putn().unwrap();
        writer.write_getc().unwrap();
        writer.write_getn().unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((super::CMD_PUSH, -1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_DUP, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_COPY, 1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_SWAP, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_DISCARD, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_SLIDE, 2)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_ADD, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_SUB, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_MUL, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_DIV, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_MOD, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_STORE, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_RETRIEVE, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_MARK, -1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_CALL, 1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_JUMP, -1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_JUMPZ, 1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_JUMPN, -1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_RETURN, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_EXIT, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_PUTC, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_PUTN, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_GETC, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_GETN, 0)));
    }

    #[test]
    fn test_assemble() {
        let mut writer = MemWriter::new();
        {
            let vec: Vec<IoResult<ir::Instruction>> = vec!(
                Ok(ir::StackPush(1)),
                Ok(ir::StackDuplicate),
                Ok(ir::StackCopy(2)),
                Ok(ir::StackSwap),
                Ok(ir::StackDiscard),
                Ok(ir::StackSlide(3)),
                Ok(ir::Addition),
                Ok(ir::Subtraction),
                Ok(ir::Multiplication),
                Ok(ir::Division),
                Ok(ir::Modulo),
                Ok(ir::HeapStore),
                Ok(ir::HeapRetrieve),
                Ok(ir::Mark(4)),
                Ok(ir::Call(5)),
                Ok(ir::Jump(6)),
                Ok(ir::JumpIfZero(7)),
                Ok(ir::JumpIfNegative(8)),
                Ok(ir::Return),
                Ok(ir::Exit),
                Ok(ir::PutCharactor),
                Ok(ir::PutNumber),
                Ok(ir::GetCharactor),
                Ok(ir::GetNumber),
                );
            let mut it = vec.move_iter();
            writer.assemble(&mut it).unwrap();
        }
        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((super::CMD_PUSH, 1)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_DUP, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_COPY, 2)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_SWAP, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_DISCARD, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_SLIDE, 3)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_ADD, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_SUB, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_MUL, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_DIV, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_MOD, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_STORE, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_RETRIEVE, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_MARK, 4)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_CALL, 5)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_JUMP, 6)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_JUMPZ, 7)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_JUMPN, 8)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_RETURN, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_EXIT, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_PUTC, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_PUTN, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_GETC, 0)));
        assert_eq!(reader.read_inst(), Ok((super::CMD_GETN, 0)));
    }

    #[test]
    fn test_disassemble() {
        let mut writer = MemWriter::new();
        writer.write_push(-1).unwrap();
        writer.write_dup().unwrap();
        writer.write_copy(1).unwrap();
        writer.write_swap().unwrap();
        writer.write_discard().unwrap();
        writer.write_slide(2).unwrap();
        writer.write_add().unwrap();
        writer.write_sub().unwrap();
        writer.write_mul().unwrap();
        writer.write_div().unwrap();
        writer.write_mod().unwrap();
        writer.write_store().unwrap();
        writer.write_retrieve().unwrap();
        writer.write_mark(-1).unwrap();
        writer.write_call(1).unwrap();
        writer.write_jump(-1).unwrap();
        writer.write_jumpz(1).unwrap();
        writer.write_jumpn(-1).unwrap();
        writer.write_return().unwrap();
        writer.write_exit().unwrap();
        writer.write_putc().unwrap();
        writer.write_putn().unwrap();
        writer.write_getc().unwrap();
        writer.write_getn().unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        let mut it = reader.disassemble();
        assert_eq!(it.next().unwrap(), Ok(ir::StackPush(-1)));
        assert_eq!(it.next().unwrap(), Ok(ir::StackDuplicate));
        assert_eq!(it.next().unwrap(), Ok(ir::StackCopy(1)));
        assert_eq!(it.next().unwrap(), Ok(ir::StackSwap));
        assert_eq!(it.next().unwrap(), Ok(ir::StackDiscard));
        assert_eq!(it.next().unwrap(), Ok(ir::StackSlide(2)));
        assert_eq!(it.next().unwrap(), Ok(ir::Addition));
        assert_eq!(it.next().unwrap(), Ok(ir::Subtraction));
        assert_eq!(it.next().unwrap(), Ok(ir::Multiplication));
        assert_eq!(it.next().unwrap(), Ok(ir::Division));
        assert_eq!(it.next().unwrap(), Ok(ir::Modulo));
        assert_eq!(it.next().unwrap(), Ok(ir::HeapStore));
        assert_eq!(it.next().unwrap(), Ok(ir::HeapRetrieve));
        assert_eq!(it.next().unwrap(), Ok(ir::Mark(-1)));
        assert_eq!(it.next().unwrap(), Ok(ir::Call(1)));
        assert_eq!(it.next().unwrap(), Ok(ir::Jump(-1)));
        assert_eq!(it.next().unwrap(), Ok(ir::JumpIfZero(1)));
        assert_eq!(it.next().unwrap(), Ok(ir::JumpIfNegative(-1)));
        assert_eq!(it.next().unwrap(), Ok(ir::Return));
        assert_eq!(it.next().unwrap(), Ok(ir::Exit));
        assert_eq!(it.next().unwrap(), Ok(ir::PutCharactor));
        assert_eq!(it.next().unwrap(), Ok(ir::PutNumber));
        assert_eq!(it.next().unwrap(), Ok(ir::GetCharactor));
        assert_eq!(it.next().unwrap(), Ok(ir::GetNumber));
        assert!(it.next().is_none());
    }
}
