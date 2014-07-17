//! Bytecode utilities.

use std::io::IoResult;

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

#[allow(missing_doc)] // FIXME
pub trait ByteCodeWriter {
    fn write_push(&mut self, n: i64) -> IoResult<()>;
    fn write_dup(&mut self) -> IoResult<()>;
    fn write_copy(&mut self, n: i64) -> IoResult<()>;
    fn write_swap(&mut self) -> IoResult<()>;
    fn write_discard(&mut self) -> IoResult<()>;
    fn write_slide(&mut self, n: i64) -> IoResult<()>;
    fn write_add(&mut self) -> IoResult<()>;
    fn write_sub(&mut self) -> IoResult<()>;
    fn write_mul(&mut self) -> IoResult<()>;
    fn write_div(&mut self) -> IoResult<()>;
    fn write_mod(&mut self) -> IoResult<()>;
    fn write_store(&mut self) -> IoResult<()>;
    fn write_retrieve(&mut self) -> IoResult<()>;
    fn write_mark(&mut self, n: i64) -> IoResult<()>;
    fn write_call(&mut self, n: i64) -> IoResult<()>;
    fn write_jump(&mut self, n: i64) -> IoResult<()>;
    fn write_jumpz(&mut self, n: i64) -> IoResult<()>;
    fn write_jumpn(&mut self, n: i64) -> IoResult<()>;
    fn write_return(&mut self) -> IoResult<()>;
    fn write_exit(&mut self) -> IoResult<()>;
    fn write_putn(&mut self) -> IoResult<()>;
    fn write_putc(&mut self) -> IoResult<()>;
    fn write_getc(&mut self) -> IoResult<()>;
    fn write_getn(&mut self) -> IoResult<()>;
}

impl<W: Writer> ByteCodeWriter for W {
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

/// Bytecodes reader.
pub trait ByteCodeReader: Reader + Seek {
    /// Read instruction bytes.
    fn read_inst(&mut self) -> IoResult<(u8, i64)>;
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
    use std::io::{MemReader, MemWriter};
    use super::*;

    #[test]
    fn test_stack() {
        let mut writer = MemWriter::new();
        writer.write_push(-1).unwrap();
        writer.write_dup().unwrap();
        writer.write_copy(1).unwrap();
        writer.write_swap().unwrap();
        writer.write_discard().unwrap();
        writer.write_slide(2).unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((CMD_PUSH, -1)));
        assert_eq!(reader.read_inst(), Ok((CMD_DUP, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_COPY, 1)));
        assert_eq!(reader.read_inst(), Ok((CMD_SWAP, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_DISCARD, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_SLIDE, 2)));
    }

    #[test]
    fn test_arithmetic() {
        let mut writer = MemWriter::new();
        writer.write_add().unwrap();
        writer.write_sub().unwrap();
        writer.write_mul().unwrap();
        writer.write_div().unwrap();
        writer.write_mod().unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((CMD_ADD, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_SUB, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_MUL, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_DIV, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_MOD, 0)));
    }

    #[test]
    fn test_heap() {
        let mut writer = MemWriter::new();
        writer.write_store().unwrap();
        writer.write_retrieve().unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((CMD_STORE, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_RETRIEVE, 0)));
    }

    #[test]
    fn test_flow() {
        let mut writer = MemWriter::new();
        writer.write_mark(-1).unwrap();
        writer.write_call(1).unwrap();
        writer.write_jump(-1).unwrap();
        writer.write_jumpz(1).unwrap();
        writer.write_jumpn(-1).unwrap();
        writer.write_return().unwrap();
        writer.write_exit().unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((CMD_MARK, -1)));
        assert_eq!(reader.read_inst(), Ok((CMD_CALL, 1)));
        assert_eq!(reader.read_inst(), Ok((CMD_JUMP, -1)));
        assert_eq!(reader.read_inst(), Ok((CMD_JUMPZ, 1)));
        assert_eq!(reader.read_inst(), Ok((CMD_JUMPN, -1)));
        assert_eq!(reader.read_inst(), Ok((CMD_RETURN, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_EXIT, 0)));
    }

    #[test]
    fn test_io() {
        let mut writer = MemWriter::new();
        writer.write_putc().unwrap();
        writer.write_putn().unwrap();
        writer.write_getc().unwrap();
        writer.write_getn().unwrap();

        let mut reader = MemReader::new(writer.unwrap());
        assert_eq!(reader.read_inst(), Ok((CMD_PUTC, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_PUTN, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_GETC, 0)));
        assert_eq!(reader.read_inst(), Ok((CMD_GETN, 0)));
    }
}
