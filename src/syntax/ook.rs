use std::collections::HashMap;
use std::io::{InvalidInput, IoError, IoResult};
use std::iter::count;

use bytecode::ByteCodeReader;
use syntax;
use syntax::{AST, Syntax};

static M_PTRI: i64 = -1;
static M_PTRD: i64 = -2;
static M_INC: i64 = -3;
static M_DEC: i64 = -4;
static M_PUTC: i64 = -5;
static M_GETC: i64 = -6;
static M_FAIL: i64 = -7;
static PTR: i64 = -1;

pub struct Ook;

impl Syntax for Ook {
    fn new() -> Ook { Ook }

    fn parse_str<'a>(&self, input: &'a str, output: &mut AST) -> IoResult<()> {
        let mut labels = HashMap::new();
        let mut label_counter = count(1, 1);
        let mut loop_counter = count(1, 1);
        let mut loop_stack = Vec::new();

        for pos in regex!("Ook(\\.|\\?|!) Ook(\\.|\\?|!)").find_iter(input) {
            let (start, end) = pos;
            match input.slice(start, end) {
                "Ook. Ook?" => output.push(syntax::WBCall(M_PTRI)),
                "Ook? Ook." => output.push(syntax::WBCall(M_PTRD)),
                "Ook. Ook." => output.push(syntax::WBCall(M_INC)),
                "Ook! Ook!" => output.push(syntax::WBCall(M_DEC)),
                "Ook. Ook!" => output.push(syntax::WBCall(M_GETC)),
                "Ook! Ook." => output.push(syntax::WBCall(M_PUTC)),
                "Ook! Ook?" => {
                    let l: i64 = loop_counter.next().unwrap();
                    loop_stack.push(l);
                    output.push(syntax::WBMark(self.marker(format!("{}#", l), &mut labels, &mut label_counter)));
                    output.push(syntax::WBPush(PTR));
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBRetrieve);
                    output.push(syntax::WBJumpIfZero(self.marker(format!("#{}", l), &mut labels, &mut label_counter)));
                },
                "Ook? Ook!" => {
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
                _ => unimplemented!(),
            }
        }
        output.push(syntax::WBExit);
        { // >
            output.push(syntax::WBMark(M_PTRI));
            output.push(syntax::WBPush(PTR));
            output.push(syntax::WBDuplicate);
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBPush(1));
            output.push(syntax::WBAddition);
            output.push(syntax::WBStore);
            output.push(syntax::WBReturn);
        }
        { // <
            output.push(syntax::WBMark(M_PTRD));
            output.push(syntax::WBPush(PTR));
            output.push(syntax::WBDuplicate);
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBPush(1));
            output.push(syntax::WBSubtraction);
            output.push(syntax::WBDuplicate);
            output.push(syntax::WBJumpIfNegative(M_FAIL));
            output.push(syntax::WBStore);
            output.push(syntax::WBReturn);
        }
        { // +
            output.push(syntax::WBMark(M_INC));
            output.push(syntax::WBPush(PTR));
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBDuplicate);
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBPush(1));
            output.push(syntax::WBAddition);
            output.push(syntax::WBStore);
            output.push(syntax::WBReturn);
        }
        { // -
            output.push(syntax::WBMark(M_DEC));
            output.push(syntax::WBPush(PTR));
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBDuplicate);
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBPush(1));
            output.push(syntax::WBSubtraction);
            output.push(syntax::WBStore);
            output.push(syntax::WBReturn);
        }
        { // .
            output.push(syntax::WBMark(M_PUTC));
            output.push(syntax::WBPush(PTR));
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBPutCharactor);
            output.push(syntax::WBReturn);
        }
        { // ,
            output.push(syntax::WBMark(M_GETC));
            output.push(syntax::WBPush(PTR));
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBRetrieve);
            output.push(syntax::WBGetCharactor);
            output.push(syntax::WBReturn);
        }
        output.push(syntax::WBMark(M_FAIL));
        Ok(())
    }

    fn parse<B: Buffer>(&self, input: &mut B, output: &mut AST) -> IoResult<()> {
        let source = try!(input.read_to_string());
        self.parse_str(source.as_slice(), output)
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

    #[test]
    fn test_ptr() {
        let source = "Ook. Ook? Ook? Ook.";
        let syntax: Ook = Syntax::new();
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBCall(super::M_PTRI)));
        assert_eq!(ast.shift(), Some(WBCall(super::M_PTRD)));
        assert_eq!(ast.shift(), Some(WBExit));
        assert!(ast.shift().is_some());
    }

    #[test]
    fn test_incdec() {
        let source = "Ook. Ook. Ook! Ook!";
        let syntax: Ook = Syntax::new();
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBCall(super::M_INC)));
        assert_eq!(ast.shift(), Some(WBCall(super::M_DEC)));
        assert_eq!(ast.shift(), Some(WBExit));
        assert!(ast.shift().is_some());
    }

    #[test]
    fn test_io() {
        let source = "Ook. Ook! Ook! Ook.";
        let syntax: Ook = Syntax::new();
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        assert_eq!(ast.shift(), Some(WBCall(super::M_GETC)));
        assert_eq!(ast.shift(), Some(WBCall(super::M_PUTC)));
        assert_eq!(ast.shift(), Some(WBExit));
        assert!(ast.shift().is_some());
    }

    #[test]
    fn test_loop() {
        let source = "Ook! Ook? Ook! Ook? Ook? Ook! Ook? Ook!";
        let syntax: Ook = Syntax::new();
        let mut ast = vec!();
        syntax.parse_str(source.as_slice(), &mut ast).unwrap();
        // outer loop
        assert_eq!(ast.shift(), Some(WBMark(1)));
        assert_eq!(ast.shift(), Some(WBPush(super::PTR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBJumpIfZero(2)));
        // inner loop
        assert_eq!(ast.shift(), Some(WBMark(3)));
        assert_eq!(ast.shift(), Some(WBPush(super::PTR)));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBRetrieve));
        assert_eq!(ast.shift(), Some(WBJumpIfZero(4)));
        assert_eq!(ast.shift(), Some(WBJump(3)));
        assert_eq!(ast.shift(), Some(WBMark(4)));
        // outer loop
        assert_eq!(ast.shift(), Some(WBJump(1)));
        assert_eq!(ast.shift(), Some(WBMark(2)));

        assert_eq!(ast.shift(), Some(WBExit));
        assert!(ast.shift().is_some());
    }
}
