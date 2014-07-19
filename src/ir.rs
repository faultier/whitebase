//! Intermediate representations of instruction set.

#![stable]

#[allow(missing_doc)]
#[deriving(PartialEq, Eq, Clone, Hash, Show)]
pub enum Instruction {
    StackPush(i64),
    StackDuplicate,
    StackCopy(i64),
    StackSwap,
    StackDiscard,
    StackSlide(i64),
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    HeapStore,
    HeapRetrieve,
    Mark(i64),
    Call(i64),
    Jump(i64),
    JumpIfZero(i64),
    JumpIfNegative(i64),
    Return,
    Exit,
    PutCharactor,
    PutNumber,
    GetCharactor,
    GetNumber,
}
