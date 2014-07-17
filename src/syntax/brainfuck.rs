//! Parser for Brainfuck.

use std::collections::HashMap;
use std::io::{EndOfFile, InvalidInput, IoResult, IoError, standard_error};
use std::iter::{Counter, count};

use syntax;
use syntax::{AST, Compiler};

pub static BF_FAIL_MARKER: i64 = -1;
pub static BF_PTR_ADDR: i64 = -1;

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

struct Scan<'r, T> {
    buffer: &'r mut T
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

struct Tokens<T> {
    iter: T,
}

impl<I: Iterator<IoResult<char>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let c = self.iter.next();
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

/// Parser for Brainfuck.
pub struct Parser<T> {
    iter: T,
    stack: Vec<i64>,
    lcount: Counter<i64>,
}

impl<I: Iterator<IoResult<Token>>> Parser<I> {
    /// Create a new `Parser` with token iterator.
    pub fn new(iter: I) -> Parser<I> {
        Parser {
            iter: iter,
            stack: Vec::new(),
            lcount: count(1, 1),
        }
    }

    /// Parse Brainfuck tokens.
    pub fn parse(&mut self, output: &mut AST) -> IoResult<()> {
        let mut labels = HashMap::new();
        let mut count = count(1, 1);
        let marker = |label: String| -> i64 {
            match labels.find_copy(&label) {
                Some(val) => val,
                None => {
                    let val = count.next().unwrap();
                    labels.insert(label, val);
                    val
                },
            }
        };

        for token in self.iter {
            match token {
                Ok(MoveRight) => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBAddition);
                    output.push(syntax::WBStore);
                },
                Ok(MoveLeft) => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBSubtraction);
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBJumpIfNegative(BF_FAIL_MARKER));
                    output.push(syntax::WBStore);
                },
                Ok(Increment) => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBAddition);
                    output.push(syntax::WBStore);
                },
                Ok(Decrement) => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBSubtraction);
                    output.push(syntax::WBStore);
                },
                Ok(Get) => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBGetCharactor);
                },
                Ok(Put) => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPutCharactor);
                },
                Ok(LoopStart) => {
                    let l: i64 = self.lcount.next().unwrap();
                    self.stack.push(l);
                    output.push(syntax::WBMark(marker(format!("{}#", l))));
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBJumpIfZero(marker(format!("#{}", l))));
                },
                Ok(LoopEnd) => {
                    match self.stack.pop() {
                        Some(l) => {
                            output.push(syntax::WBJump(marker(format!("{}#", l))));
                            output.push(syntax::WBMark(marker(format!("#{}", l))));
                        },
                        None => return Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("broken loop".to_string()),
                        }),
                    }
                },
                Err(e) => return Err(e),
            }
        }
        output.push(syntax::WBExit);
        output.push(syntax::WBMark(BF_FAIL_MARKER));

        Ok(())
    }
}

/// Compiler for Brainfuck.
pub struct Brainfuck;

impl Brainfuck {
    /// Create a new `Brainfuck`.
    pub fn new() -> Brainfuck { Brainfuck }
}

impl Compiler for Brainfuck {
    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
        Parser::new(Tokens { iter: Scan { buffer: input } }).parse(output)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syntax::*;
    use std::io::BufReader;

    #[test]
    fn test_scan() {
        let mut buffer = BufReader::new("><+- ,.\n[饂飩]".as_bytes());
        let mut it = super::Scan { buffer: &mut buffer };
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
        let mut it = super::Tokens { iter: super::Scan { buffer: &mut buffer } };
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
    fn test_ptr() {
        let syntax = Brainfuck::new();

        let source = ">";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPush(1)));
        assert_eq!(ast.shift(), Some(WBAddition));
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());

        let source = "<";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPush(1)));
        assert_eq!(ast.shift(), Some(WBSubtraction));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBJumpIfNegative(BF_FAIL_MARKER)));
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());

        /* TODO: optimize
        let source = ">><>>";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPush(3))); // optimized
        assert_eq!(ast.shift(), Some(WBAddition));
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());
        */
    }

    #[test]
    fn test_incdec() {
        let syntax = Brainfuck::new();

        let source = "+";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPush(1)));
        assert_eq!(ast.shift(), Some(WBAddition));
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());

        let source = "-";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPush(1)));
        assert_eq!(ast.shift(), Some(WBSubtraction));
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());

        /* TODO: optimize
        let source = "-++-+---";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBDuplicate));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPush(2))); // optimized
        assert_eq!(ast.shift(), Some(WBSubtraction));
        assert_eq!(ast.shift(), Some(WBStore));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());
        */
    }

    #[test]
    fn test_io() {
        let syntax = Brainfuck::new();

        let source = ",";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBGetCharactor));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());

        let source = ".";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBPutCharactor));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());
    }

    #[test]
    fn test_loop() {
        let source = "[[]]";
        let syntax = Brainfuck::new();
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        // outer loop
        assert_eq!(ast.shift(), Some(WBMark(1)));
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBJumpIfZero(2)));
        // inner loop
        assert_eq!(ast.shift(), Some(WBMark(3)));
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBJumpIfZero(4)));
        assert_eq!(ast.shift(), Some(WBJump(3)));
        assert_eq!(ast.shift(), Some(WBMark(4)));
        // outer loop
        assert_eq!(ast.shift(), Some(WBJump(1)));
        assert_eq!(ast.shift(), Some(WBMark(2)));

        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());
    }
}
