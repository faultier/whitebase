use std::collections::HashMap;
use std::io::{BufReader, EndOfFile, InvalidInput, IoResult, IoError};
use std::iter::count;

use bytecode::ByteCodeReader;
use syntax;
use syntax::{AST, Syntax};

pub static BF_FAIL_MARKER: i64 = -1;
pub static BF_PTR_ADDR: i64 = -1;

pub trait BrainfuckSyntax {
    fn map<B: Buffer>(&self, &mut B, |Token| -> IoResult<()>) -> IoResult<()>;
}

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

pub struct Wrapper<T> {
    inner: T
}

impl<S: BrainfuckSyntax> Wrapper<S> {
    pub fn new(inner: S) -> Wrapper<S> { Wrapper { inner: inner } }
}

impl<S: BrainfuckSyntax> Syntax for Wrapper<S> {
    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        self.parse(&mut BufReader::new(input.as_bytes()), output)
    }

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
        let mut labels = HashMap::new();
        let mut label_counter = count(1, 1);
        let mut loop_counter = count(1, 1);
        let mut loop_stack = Vec::new();

        try!(self.inner.map(input, |token| -> IoResult<()> {
            match token {
                MoveRight => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBAddition);
                    output.push(syntax::WBStore);
                },
                MoveLeft => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBSubtraction);
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBJumpIfNegative(BF_FAIL_MARKER));
                    output.push(syntax::WBStore);
                },
                Increment => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBAddition);
                    output.push(syntax::WBStore);
                },
                Decrement => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBDuplicate);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPush(1));
                    output.push(syntax::WBSubtraction);
                    output.push(syntax::WBStore);
                },
                Get => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBGetCharactor);
                },
                Put => {
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBPutCharactor);
                },
                LoopStart => {
                    let l: i64 = loop_counter.next().unwrap();
                    loop_stack.push(l);
                    output.push(syntax::WBMark(self.marker(format!("{}#", l), &mut labels, &mut label_counter)));
                    output.push(syntax::WBPush(BF_PTR_ADDR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBJumpIfZero(self.marker(format!("#{}", l), &mut labels, &mut label_counter)));
                },
                LoopEnd => {
                    match loop_stack.pop() {
                        Some(l) => {
                            output.push(syntax::WBJump(self.marker(format!("{}#", l), &mut labels, &mut label_counter)));
                            output.push(syntax::WBMark(self.marker(format!("#{}", l), &mut labels, &mut label_counter)));
                        },
                        None => return Err(IoError {
                            kind: InvalidInput,
                            desc: "syntax error",
                            detail: Some("broken loop".to_string()),
                        }),
                    }
                },
            };
            Ok(())
        }));
        output.push(syntax::WBExit);
        output.push(syntax::WBMark(BF_FAIL_MARKER));
        Ok(())
    }

    #[allow(unused_variable)]
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        unimplemented!()
    }
}

pub struct Brainfuck;

impl Brainfuck {
    pub fn new() -> Wrapper<Brainfuck> { Wrapper::new(Brainfuck) }
}

impl BrainfuckSyntax for Brainfuck {
    fn map<B: Buffer>(&self, input: &mut B, block: |Token| -> IoResult<()>) -> IoResult<()> {
        loop {
            let ret = match input.read_char() {
                Ok('>') => MoveRight,
                Ok('<') => MoveLeft,
                Ok('+') => Increment,
                Ok('-') => Decrement,
                Ok(',') => Get,
                Ok('.') => Put,
                Ok('[') => LoopStart,
                Ok(']') => LoopEnd,
                Ok(_)   => continue,
                Err(IoError { kind: EndOfFile, ..}) => return Ok(()),
                Err(e) => return Err(e),
            };
            try!(block(ret))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syntax::*;

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
