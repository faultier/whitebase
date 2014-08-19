//! Parser for Brainfuck.

#![experimental]

use std::collections::HashMap;
use std::io::{EndOfFile, InvalidInput, IoResult, IoError, standard_error};
use std::iter::{Counter, count};

use bytecode::ByteCodeWriter;
use ir;
use ir::Instruction;
use syntax::Compiler;

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
        match self.buffer.remove(0) {
            Some(i) => Some(i),
            None => {
                let ret = match self.tokens.next() {
                    Some(Ok(MoveRight)) => vec!(
                        Ok(ir::StackPush(BF_PTR_ADDR)),
                        Ok(ir::StackDuplicate),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::StackPush(1)),
                        Ok(ir::Addition),
                        Ok(ir::HeapStore),
                    ),
                    Some(Ok(MoveLeft)) => vec!(
                        Ok(ir::StackPush(BF_PTR_ADDR)),
                        Ok(ir::StackDuplicate),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::StackPush(1)),
                        Ok(ir::Subtraction),
                        Ok(ir::StackDuplicate),
                        Ok(ir::JumpIfNegative(BF_FAIL_MARKER)),
                        Ok(ir::HeapStore),
                    ),
                    Some(Ok(Increment)) => vec!(
                        Ok(ir::StackPush(BF_PTR_ADDR)),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::StackDuplicate),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::StackPush(1)),
                        Ok(ir::Addition),
                        Ok(ir::HeapStore),
                    ),
                    Some(Ok(Decrement)) => vec!(
                        Ok(ir::StackPush(BF_PTR_ADDR)),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::StackDuplicate),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::StackPush(1)),
                        Ok(ir::Subtraction),
                        Ok(ir::HeapStore),
                    ),
                    Some(Ok(Get)) => vec!(
                        Ok(ir::StackPush(BF_PTR_ADDR)),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::GetCharactor),
                    ),
                    Some(Ok(Put)) => vec!(
                        Ok(ir::StackPush(BF_PTR_ADDR)),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::HeapRetrieve),
                        Ok(ir::PutCharactor),
                    ),
                    Some(Ok(LoopStart)) => {
                        let l: i64 = self.scount.next().unwrap();
                        self.stack.push(l);
                        vec!(
                            Ok(ir::Mark(self.marker(format!("{}#", l)))),
                            Ok(ir::StackPush(BF_PTR_ADDR)),
                            Ok(ir::HeapRetrieve),
                            Ok(ir::HeapRetrieve),
                            Ok(ir::JumpIfZero(self.marker(format!("#{}", l)))),
                        )
                    }
                    Some(Ok(LoopEnd)) => {
                        match self.stack.pop() {
                            Some(l) => vec!(
                                Ok(ir::Jump(self.marker(format!("{}#", l)))),
                                Ok(ir::Mark(self.marker(format!("#{}", l)))),
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
                        vec!(Ok(ir::Exit), Ok(ir::Mark(BF_FAIL_MARKER)))
                    }
                };
                self.buffer.push_all(ret.as_slice());
                self.buffer.remove(0)
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

impl Compiler for Brainfuck {
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut it = scan(input).tokenize().parse();
        output.assemble(&mut it)
    }
}

#[cfg(test)]
mod test {
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
        assert_eq!(it.next(), Some(Ok(super::MoveRight)));
        assert_eq!(it.next(), Some(Ok(super::MoveLeft)));
        assert_eq!(it.next(), Some(Ok(super::Increment)));
        assert_eq!(it.next(), Some(Ok(super::Decrement)));
        assert_eq!(it.next(), Some(Ok(super::Get)));
        assert_eq!(it.next(), Some(Ok(super::Put)));
        assert_eq!(it.next(), Some(Ok(super::LoopStart)));
        assert_eq!(it.next(), Some(Ok(super::LoopEnd)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_parse() {
        let mut buffer = BufReader::new(">".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(StackDuplicate)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(StackPush(1))));
        assert_eq!(it.next(), Some(Ok(Addition)));
        assert_eq!(it.next(), Some(Ok(HeapStore)));
        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("<".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(StackDuplicate)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(StackPush(1))));
        assert_eq!(it.next(), Some(Ok(Subtraction)));
        assert_eq!(it.next(), Some(Ok(StackDuplicate)));
        assert_eq!(it.next(), Some(Ok(JumpIfNegative(super::BF_FAIL_MARKER))));
        assert_eq!(it.next(), Some(Ok(HeapStore)));
        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("+".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(StackDuplicate)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(StackPush(1))));
        assert_eq!(it.next(), Some(Ok(Addition)));
        assert_eq!(it.next(), Some(Ok(HeapStore)));
        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("-".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(StackDuplicate)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(StackPush(1))));
        assert_eq!(it.next(), Some(Ok(Subtraction)));
        assert_eq!(it.next(), Some(Ok(HeapStore)));
        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new(",".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(GetCharactor)));
        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new(".".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(PutCharactor)));
        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());

        let mut buffer = BufReader::new("[[]]".as_bytes());
        let mut it = super::scan(&mut buffer).tokenize().parse();
        // outer loop
        assert_eq!(it.next(), Some(Ok(Mark(1))));
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(JumpIfZero(2))));
        // inner loop
        assert_eq!(it.next(), Some(Ok(Mark(3))));
        assert_eq!(it.next(), Some(Ok(StackPush(super::BF_PTR_ADDR))));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(HeapRetrieve)));
        assert_eq!(it.next(), Some(Ok(JumpIfZero(4))));
        assert_eq!(it.next(), Some(Ok(Jump(3))));
        assert_eq!(it.next(), Some(Ok(Mark(4))));
        // outer loop
        assert_eq!(it.next(), Some(Ok(Jump(1))));
        assert_eq!(it.next(), Some(Ok(Mark(2))));

        assert_eq!(it.next(), Some(Ok(Exit)));
        assert_eq!(it.next(), Some(Ok(Mark(super::BF_FAIL_MARKER))));
        assert!(it.next().is_none());
    }
}
