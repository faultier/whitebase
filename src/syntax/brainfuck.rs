//! Parser for Brainfuck.

#![experimental]

use std::collections::HashMap;
use std::io::{EndOfFile, InvalidInput, IoResult, IoError, standard_error};
use std::iter::{Counter, count};

use bytecode::ByteCodeWriter;
use ir;
use ir::Instruction;
use syntax::Compile;

pub static BF_FAIL_MARKER: i64 = -1;
pub static BF_PTR_ADDR: i64 = -1;

/// An iterator that convert to IR from brainfuck tokens on each iteration.
pub struct Instructions<T> {
    tokens: T,
    stack: Vec<i64>,
    scount: Counter<i64>,
    labels: HashMap<String, i64>,
    lcount: Counter<i64>,
    buffer: Vec<IoResult<Instruction>>,
    parsed: bool,
}

impl<I: Iterator<IoResult<Token>>> Instructions<I> {
    /// Create an iterator that convert to IR from tokens on each iteration.
    pub fn new(iter: I) -> Instructions<I> {
        Instructions {
            tokens: iter,
            stack: Vec::new(),
            scount: count(1, 1),
            labels: HashMap::new(),
            lcount: count(1, 1),
            buffer: Vec::new(),
            parsed: false,
        }
    }

    fn marker(&mut self, label: String) -> i64 {
        match self.labels.find_copy(&label) {
            Some(val) => val,
            None => {
                let val = self.lcount.next().unwrap();
                self.labels.insert(label, val);
                val
            },
        }
    }
}

impl<I: Iterator<IoResult<Token>>> Iterator<IoResult<Instruction>> for Instructions<I> {
    fn next(&mut self) -> Option<IoResult<Instruction>> {
        match self.buffer.shift() {
            Some(i) => Some(i),
            None => {
                let ret = match self.tokens.next() {
                    Some(Ok(MoveRight)) => vec!(
                        Ok(ir::WBPush(BF_PTR_ADDR)),
                        Ok(ir::WBDuplicate),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBPush(1)),
                        Ok(ir::WBAddition),
                        Ok(ir::WBStore),
                    ),
                    Some(Ok(MoveLeft)) => vec!(
                        Ok(ir::WBPush(BF_PTR_ADDR)),
                        Ok(ir::WBDuplicate),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBPush(1)),
                        Ok(ir::WBSubtraction),
                        Ok(ir::WBDuplicate),
                        Ok(ir::WBJumpIfNegative(BF_FAIL_MARKER)),
                        Ok(ir::WBStore),
                    ),
                    Some(Ok(Increment)) => vec!(
                        Ok(ir::WBPush(BF_PTR_ADDR)),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBDuplicate),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBPush(1)),
                        Ok(ir::WBAddition),
                        Ok(ir::WBStore),
                    ),
                    Some(Ok(Decrement)) => vec!(
                        Ok(ir::WBPush(BF_PTR_ADDR)),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBDuplicate),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBPush(1)),
                        Ok(ir::WBSubtraction),
                        Ok(ir::WBStore),
                    ),
                    Some(Ok(Get)) => vec!(
                        Ok(ir::WBPush(BF_PTR_ADDR)),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBGetCharactor),
                    ),
                    Some(Ok(Put)) => vec!(
                        Ok(ir::WBPush(BF_PTR_ADDR)),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBRetrieve),
                        Ok(ir::WBPutCharactor),
                    ),
                    Some(Ok(LoopStart)) => {
                        let l: i64 = self.scount.next().unwrap();
                        self.stack.push(l);
                        vec!(
                            Ok(ir::WBMark(self.marker(format!("{}#", l)))),
                            Ok(ir::WBPush(BF_PTR_ADDR)),
                            Ok(ir::WBRetrieve),
                            Ok(ir::WBRetrieve),
                            Ok(ir::WBJumpIfZero(self.marker(format!("#{}", l)))),
                        )
                    }
                    Some(Ok(LoopEnd)) => {
                        match self.stack.pop() {
                            Some(l) => vec!(
                                Ok(ir::WBJump(self.marker(format!("{}#", l)))),
                                Ok(ir::WBMark(self.marker(format!("#{}", l)))),
                            ),
                            None => vec!(
                                Err(IoError {
                                    kind: InvalidInput,
                                    desc: "syntax error",
                                    detail: Some("broken loop".to_string()),
                                })
                            ),
                        }
                    }
                    Some(Err(e)) => vec!(Err(e)),
                    None => {
                        if self.parsed { return None }
                        self.parsed = true;
                        vec!(Ok(ir::WBExit), Ok(ir::WBMark(BF_FAIL_MARKER)))
                    }
                };
                self.buffer.push_all(ret.as_slice());
                self.buffer.shift()
            }
        }
    }
}

#[allow(missing_doc)]
#[deriving(PartialEq, Show)]
pub enum Token {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    Put,
    Get,
    LoopStart,
    LoopEnd,
}

struct Tokens<T> {
    lexemes: T,
}

impl<I: Iterator<IoResult<char>>> Tokens<I> {
    pub fn parse(self) -> Instructions<Tokens<I>> { Instructions::new(self) }
}

impl<I: Iterator<IoResult<char>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let c = self.lexemes.next();
        if c.is_none() { return None; }

        Some(match c.unwrap() {
            Ok('>') => Ok(MoveRight),
            Ok('<') => Ok(MoveLeft),
            Ok('+') => Ok(Increment),
            Ok('-') => Ok(Decrement),
            Ok(',') => Ok(Get),
            Ok('.') => Ok(Put),
            Ok('[') => Ok(LoopStart),
            Ok(']') => Ok(LoopEnd),
            Ok(_)   => Err(standard_error(InvalidInput)),
            Err(e)  => Err(e),
        })
    }
}

struct Scan<'r, T> {
    buffer: &'r mut T
}

impl<'r, B: Buffer> Scan<'r, B> {
    pub fn tokenize(self) -> Tokens<Scan<'r, B>> { Tokens { lexemes: self } }
}

impl<'r, B: Buffer> Iterator<IoResult<char>> for Scan<'r, B> {
    fn next(&mut self) -> Option<IoResult<char>> {
        loop {
            let ret = match self.buffer.read_char() {
                Ok('>') => '>',
                Ok('<') => '<',
                Ok('+') => '+',
                Ok('-') => '-',
                Ok(',') => ',',
                Ok('.') => '.',
                Ok('[') => '[',
                Ok(']') => ']',
                Ok(_)   => continue,
                Err(IoError { kind: EndOfFile, ..}) => return None,
                Err(e) => return Some(Err(e)),
            };
            return Some(Ok(ret));
        }
    }
}

fn scan<'r, B: Buffer>(buffer: &'r mut B) -> Scan<'r, B> { Scan { buffer: buffer } }

/// Compiler for Brainfuck.
pub struct Brainfuck;

impl Brainfuck {
    /// Create a new `Brainfuck`.
    pub fn new() -> Brainfuck { Brainfuck }
}

impl Compile for Brainfuck {
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut it = scan(input).tokenize().parse();
        output.assemble(&mut it)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ir::*;
    use std::io::BufReader;

    #[test]
    fn test_scan() {
        let mut buffer = BufReader::new("><+- ,.\n[饂飩]".as_bytes());
        let mut it = super::scan(&mut buffer);
        assert_eq!(it.next(), Some(Ok('>')));
        assert_eq!(it.next(), Some(Ok('<')));
        assert_eq!(it.next(), Some(Ok('+')));
        assert_eq!(it.next(), Some(Ok('-')));
        assert_eq!(it.next(), Some(Ok(',')));
        assert_eq!(it.next(), Some(Ok('.')));
        assert_eq!(it.next(), Some(Ok('[')));
        assert_eq!(it.next(), Some(Ok(']')));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_tokenize() {
        let mut buffer = BufReader::new("><+- ,.\n[饂飩]".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize();
        assert_eq!(it.next(), Some(Ok(MoveRight)));
        assert_eq!(it.next(), Some(Ok(MoveLeft)));
        assert_eq!(it.next(), Some(Ok(Increment)));
        assert_eq!(it.next(), Some(Ok(Decrement)));
        assert_eq!(it.next(), Some(Ok(Get)));
        assert_eq!(it.next(), Some(Ok(Put)));
        assert_eq!(it.next(), Some(Ok(LoopStart)));
        assert_eq!(it.next(), Some(Ok(LoopEnd)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_parse() {
        let mut buffer = BufReader::new(">".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBDuplicate)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBPush(1))));
        assert_eq!(it.next(), Some(Ok(WBAddition)));
        assert_eq!(it.next(), Some(Ok(WBStore)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("<".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBDuplicate)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBPush(1))));
        assert_eq!(it.next(), Some(Ok(WBSubtraction)));
        assert_eq!(it.next(), Some(Ok(WBDuplicate)));
        assert_eq!(it.next(), Some(Ok(WBJumpIfNegative(BF_FAIL_MARKER))));
        assert_eq!(it.next(), Some(Ok(WBStore)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("+".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBDuplicate)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBPush(1))));
        assert_eq!(it.next(), Some(Ok(WBAddition)));
        assert_eq!(it.next(), Some(Ok(WBStore)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("-".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBDuplicate)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBPush(1))));
        assert_eq!(it.next(), Some(Ok(WBSubtraction)));
        assert_eq!(it.next(), Some(Ok(WBStore)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new(",".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBGetCharactor)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new(".".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBPutCharactor)));
        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("[[]]".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        // outer loop
        assert_eq!(it.next(), Some(Ok(WBMark(1))));
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBJumpIfZero(2))));
        // inner loop
        assert_eq!(it.next(), Some(Ok(WBMark(3))));
        assert_eq!(it.next(), Some(Ok(WBPush(BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBRetrieve)));
        assert_eq!(it.next(), Some(Ok(WBJumpIfZero(4))));
        assert_eq!(it.next(), Some(Ok(WBJump(3))));
        assert_eq!(it.next(), Some(Ok(WBMark(4))));
        // outer loop
        assert_eq!(it.next(), Some(Ok(WBJump(1))));
        assert_eq!(it.next(), Some(Ok(WBMark(2))));

        assert_eq!(it.next(), Some(Ok(WBExit)));
        assert_eq!(it.next(), Some(Ok(WBMark(BF_FAIL_MARKER))));
        assert!(it.next().is_none());
    }
}
