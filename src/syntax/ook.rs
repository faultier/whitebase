use std::io::{EndOfFile, IoError, IoResult};

use syntax::brainfuck::{BrainfuckSyntax, Token, Wrapper};
use bf = syntax::brainfuck;

pub struct Ook;

impl Ook {
    pub fn new() -> Wrapper<Ook> { Wrapper::new(Ook) }
}

impl BrainfuckSyntax for Ook {
    fn map<B: Buffer>(&self, input: &mut B, block: |Token| -> IoResult<()>) -> IoResult<()> {
        loop {
            match input.read_line() {
                Ok(line) => {
                    let source = line.as_slice();
                    for pos in regex!("Ook(\\.|\\?|!) Ook(\\.|\\?|!)").find_iter(source) {
                        let (start, end) = pos;
                        let ret = match source.slice(start, end) {
                            "Ook. Ook?" => bf::MoveRight,
                            "Ook? Ook." => bf::MoveLeft,
                            "Ook. Ook." => bf::Increment,
                            "Ook! Ook!" => bf::Decrement,
                            "Ook. Ook!" => bf::Get,
                            "Ook! Ook." => bf::Put,
                            "Ook! Ook?" => bf::LoopStart,
                            "Ook? Ook!" => bf::LoopEnd,
                            _ => unimplemented!(),
                        };
                        try!(block(ret));
                    }
                },
                Err(IoError { kind: EndOfFile, ..}) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syntax::*;
    use syntax::brainfuck::*;

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
