//! Intermediate representations of instruction set.

#![experimental]

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
