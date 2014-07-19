//! Compilers and Decompilers.

#![experimental]

pub use self::assembly::Assembly;
pub use self::brainfuck::Brainfuck;
pub use self::dt::DT;
pub use self::ook::Ook;
pub use self::whitespace::Whitespace;

use std::io::IoResult;
use bytecode::{ByteCodeWriter, ByteCodeReader};

/// Convert from source code to bytecodes.
pub trait Compiler {
    /// Convert from source code to bytecodes.
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, &mut B, &mut W) -> IoResult<()>;
}

/// Generate source code from bytecods.
pub trait Decompiler {
    /// Generate source code from bytecods.
    fn decompile<R: ByteCodeReader, W: Writer>(&self, &mut R, &mut W) -> IoResult<()>;
}

pub mod assembly;
pub mod brainfuck;
pub mod dt;
pub mod ook;
pub mod whitespace;
