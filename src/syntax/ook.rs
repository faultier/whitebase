//! Parser for Ook!

#![experimental]

use std::io::{EndOfFile, InvalidInput, IoError, IoResult, standard_error};
use std::str::from_utf8;

use bytecode::ByteCodeWriter;
use syntax::Compile;
use syntax::brainfuck::{Instructions, Token, MoveRight, MoveLeft, Increment, Decrement, Put, Get, LoopStart, LoopEnd};

struct Tokens<T> {
    lexemes: T,
}

impl<I: Iterator<IoResult<String>>> Tokens<I> {
    pub fn parse(self) -> Instructions<Tokens<I>> { Instructions::new(self) }
}

impl<I: Iterator<IoResult<String>>> Iterator<IoResult<Token>> for Tokens<I> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let op = self.lexemes.next();
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

impl<'r, B: Buffer> Scan<'r, B> {
    pub fn tokenize(self) -> Tokens<Scan<'r, B>> { Tokens { lexemes: self } }
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

fn scan<'r, B: Buffer>(buffer: &'r mut B) -> Scan<'r, B> {
    Scan { buffer: buffer, is_start: true }
}

/// Compiler for Ook!.
pub struct Ook;

impl Ook {
    /// Create a new `Ook`.
    pub fn new() -> Ook { Ook }
}

impl Compile for Ook {
    fn compile<B: Buffer, W: ByteCodeWriter>(&self, input: &mut B, output: &mut W) -> IoResult<()> {
        let mut it = scan(input).tokenize().parse();
        output.assemble(&mut it)
    }
}

#[cfg(test)]
mod test {
    use syntax::brainfuck::*;
    use std::io::BufReader;

    #[test]
    fn test_scan() {
        let mut buffer = BufReader::new("Ook? Ook. Ook! Ook.\nOok. Ook? Ook.".as_bytes());
        let mut it = super::scan(&mut buffer);
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
}
