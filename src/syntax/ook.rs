use std::io::{BufReader, EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::str::from_utf8;

use bytecode::ByteCodeReader;
use syntax::{AST, Syntax};
use syntax::brainfuck::{Parser, Token, MoveRight, MoveLeft, Increment, Decrement, Put, Get, LoopStart, LoopEnd};

fn is_whitespace(c: &char) -> bool {
    *c == ' ' || is_linebreak(c)
}

fn is_linebreak(c: &char) -> bool {
    *c == '\n' || *c == '\r'
}

struct Scan<'r, T> {
    buffer: &'r mut T,
    is_start: bool,
}

impl<'r, B: Buffer> Iterator<IoResult<String>> for Scan<'r, B> {
    fn next(&mut self) -> Option<IoResult<String>> {
        let mut buf = [0u8, ..9];

        if !self.is_start {
            // skip separator
            match self.buffer.read_char() {
                Ok(ref c) if is_whitespace(c) => (),
                Ok(_) => return Some(Err(standard_error(InvalidInput))),
                Err(IoError { kind: EndOfFile, ..}) => return None,
                Err(e) => return Some(Err(e)),
            }
            // skip linebreak
            loop {
                match self.buffer.read_char() {
                    Ok(ref c) if is_linebreak(c) => continue,
                    Ok(c) => {
                        buf[0] = c as u8;
                        break;
                    },
                    Err(IoError { kind: EndOfFile, ..}) => return None,
                    Err(e) => return Some(Err(e)),
                }
            }
            match self.buffer.read(buf.mut_slice_from(1)) {
                Ok(n) if n == 8 => (),
                Ok(_)  => return Some(Err(standard_error(InvalidInput))),
                Err(IoError { kind: EndOfFile, ..}) => return None,
                Err(e) => return Some(Err(e)),
            }
        } else {
            match self.buffer.read(buf) {
                Ok(n) if n == 9 => (),
                Ok(_) => return Some(Err(standard_error(InvalidInput))),
                Err(IoError { kind: EndOfFile, ..}) => return None,
                Err(e) => return Some(Err(e)),
            }
            self.is_start = false;
        }

        match from_utf8(buf) {
            Some(string) => Some(Ok(String::from_str(string))),
            None => Some(Err(standard_error(InvalidInput))),
        }
    }
}

struct Tokens<T> {
    iter: T,
}

impl<I: Iterator<IoResult<String>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let op = self.iter.next();
        if op.is_none() { return None; }

        let res = op.unwrap();
         match res {
             Err(e) => return Some(Err(e)),
             Ok(_) => (),
        }

        Some(match res.unwrap().as_slice() {
            "Ook. Ook?" => Ok(MoveRight),
            "Ook? Ook." => Ok(MoveLeft),
            "Ook. Ook." => Ok(Increment),
            "Ook! Ook!" => Ok(Decrement),
            "Ook. Ook!" => Ok(Get),
            "Ook! Ook." => Ok(Put),
            "Ook! Ook?" => Ok(LoopStart),
            "Ook? Ook!" => Ok(LoopEnd),
            _ => Err(standard_error(InvalidInput)),
        })
    }
}

pub struct Ook;

impl Ook {
    pub fn new() -> Ook { Ook }
}

impl Syntax for Ook {
    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        self.parse(&mut BufReader::new(input.as_bytes()), output)
    }

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
        Parser::new(Tokens { iter: Scan { buffer: input, is_start: true } }).parse(output)
    }

    #[allow(unused_variable)]
    fn decompile<R: ByteCodeReader, W: Writer>(&self, input: &mut R, output: &mut W) -> IoResult<()> {
        unimplemented!()
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use syntax::*;
    use syntax::brainfuck::*;
    use std::io::BufReader;

    #[test]
    fn test_scan() {
        let mut buffer = BufReader::new("Ook? Ook. Ook! Ook.\nOok. Ook? Ook.".as_bytes());
        let mut it = super::Scan { buffer: &mut buffer, is_start: true };
        assert_eq!(it.next(), Some(Ok("Ook? Ook.".to_string())));
        assert_eq!(it.next(), Some(Ok("Ook! Ook.".to_string())));
        assert_eq!(it.next(), Some(Ok("Ook. Ook?".to_string())));
        assert!(it.next().unwrap().is_err());
    }

    #[test]
    fn test_tokenize() {
        let source = vec!(
            "Ook. Ook?",
            "Ook? Ook.",
            "Ook. Ook.",
            "Ook! Ook!",
            "Ook. Ook!",
            "Ook! Ook.",
            "Ook! Ook?",
            "Ook? Ook!",
            ).connect(" ");
        let mut buffer = BufReader::new(source.as_slice().as_bytes());
        let mut it = super::Tokens { iter: super::Scan { buffer: &mut buffer, is_start: true } };
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
        let syntax = Ook::new();

        let source = "Ook. Ook?";
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

        let source = "Ook? Ook.";
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
    }

    #[test]
    fn test_incdec() {
        let syntax = Ook::new();

        let source = "Ook. Ook.";
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

        let source = "Ook! Ook!";
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
    }

    #[test]
    fn test_io() {
        let syntax = Ook::new();

        let source = "Ook. Ook!";
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBPush(BF_PTR_ADDR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBGetCharactor));
        assert_eq!(ast.shift(), Some(WBExit));
        assert_eq!(ast.shift(), Some(WBMark(BF_FAIL_MARKER)));
        assert!(ast.shift().is_none());

        let source = "Ook! Ook.";
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
        let source = "Ook! Ook? Ook! Ook? Ook? Ook! Ook? Ook!";
        let syntax = Ook::new();
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
