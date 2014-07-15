use std::collections::HashMap;
use std::collections::TreeMap;
use std::io::{BufferedReader, EndOfFile, InvalidInput, IoError, MemReader, MemWriter, SeekSet, standard_error};
use std::io::stdio::{StdReader, StdWriter, stdin, stdout_raw};
use bc = bytecode;
use bytecode::ByteCodeReader;
use syntax::Syntax;

pub type MachineResult<T> = Result<T, MachineError>;

#[deriving(PartialEq, Show)]
pub enum MachineError {
    IllegalStackManipulation,
    UndefinedLabel,
    ZeroDivision,
    CallStackEmpty,
    MissingExitInstruction,
    MachineIoError(IoError),
    OtherMachineError,
}

pub struct Machine {
    stack: Vec<i64>,
    heap: TreeMap<i64, i64>,
    stdin: BufferedReader<StdReader>,
    stdout: StdWriter,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            stack: Vec::new(),
            heap: TreeMap::new(),
            stdin: stdin(),
            stdout: stdout_raw(),
        }
    }

    pub fn run(&mut self, program: &mut ByteCodeReader) -> MachineResult<()> {
        let mut index = HashMap::new();
        let mut caller = vec!();
        loop {
            match self.step(program, &mut index, &mut caller) {
                Err(e)    => return Err(e),
                Ok(false) => return Ok(()),
                Ok(true)  => continue,
            }
        }
    }

    fn step(&mut self, program: &mut ByteCodeReader, index: &mut HashMap<i64, u64>, caller: &mut Vec<u64>) -> MachineResult<bool> {
        match program.read_inst() {
            Ok((bc::CMD_PUSH, n))       => { try!(self.push(n)); Ok(true) },
            Ok((bc::CMD_DUP, _))        => { try!(self.copy(0)); Ok(true) },
            Ok((bc::CMD_COPY, n))       => { try!(self.copy(n.to_uint().unwrap())); Ok(true) },
            Ok((bc::CMD_SWAP, _))       => { try!(self.swap()); Ok(true) },
            Ok((bc::CMD_DISCARD, _))    => { try!(self.discard()); Ok(true) },
            Ok((bc::CMD_SLIDE, n))      => { try!(self.slide(n.to_uint().unwrap())); Ok(true) },
            Ok((bc::CMD_ADD, _))        => { try!(self.calc(|x, y| { y + x })); Ok(true) },
            Ok((bc::CMD_SUB, _))        => { try!(self.calc(|x, y| { y - x })); Ok(true) },
            Ok((bc::CMD_MUL, _))        => { try!(self.calc(|x, y| { y * x })); Ok(true) },
            Ok((bc::CMD_DIV, _))        => { try!(self.dcalc(|x, y| { y / x })); Ok(true) },
            Ok((bc::CMD_MOD, _))        => { try!(self.dcalc(|x, y| { y % x })); Ok(true) },
            Ok((bc::CMD_STORE, _))      => { try!(self.store()); Ok(true) },
            Ok((bc::CMD_RETRIEVE, _))   => { try!(self.retrieve()); Ok(true) },
            Ok((bc::CMD_MARK, _))       => Ok(true),
            Ok((bc::CMD_CALL, n))       => { try!(self.call(program, index, caller, &n)); Ok(true) },
            Ok((bc::CMD_JUMP, n))       => { try!(self.jump(program, index, &n)); Ok(true) },
            Ok((bc::CMD_JUMPZ, n))      => { try!(self.jump_if(program, index, &n, |x| { x == 0 })); Ok(true) },
            Ok((bc::CMD_JUMPN, n))      => { try!(self.jump_if(program, index, &n, |x| { x < 0 })); Ok(true) },
            Ok((bc::CMD_RETURN, _))     => { try!(self.do_return(program, caller)); Ok(true) },
            Ok((bc::CMD_EXIT, _))       => Ok(false),
            Ok((bc::CMD_PUTC, _))       => { try!(self.put_char()); Ok(true) },
            Ok((bc::CMD_PUTN, _))       => { try!(self.put_num()); Ok(true) },
            Ok((bc::CMD_GETC, _))       => { try!(self.get_char()); Ok(true) },
            Ok((bc::CMD_GETN, _))       => { try!(self.get_num()); Ok(true) },
            Err(ref e) if e.kind == EndOfFile => Err(MissingExitInstruction),
            Err(e)                      => Err(MachineIoError(e)),
            _                           => Err(OtherMachineError),
        }
    }

    fn push(&mut self, n: i64) -> MachineResult<()> {
        self.stack.push(n);
        Ok(())
    }

    fn copy(&mut self, n: uint) -> MachineResult<()> {
        if self.stack.len() <= n {
            return Err(IllegalStackManipulation)
        }
        let mut i = 0;
        let mut tmp = vec!();
        while i < n {
            tmp.unshift(self.stack.pop().unwrap());
            i += 1;
        }
        let val = self.stack.pop().unwrap();
        self.stack.push(val);
        self.stack.push_all(tmp.as_slice());
        self.stack.push(val);
        Ok(())
    }

    fn swap(&mut self) -> MachineResult<()> {
        match self.stack.pop() {
            None => Err(IllegalStackManipulation),
            Some(x) => match self.stack.pop() {
                None => Err(IllegalStackManipulation),
                Some(y) => {
                    self.stack.push(x);
                    self.stack.push(y);
                    Ok(())
                },
            },
        }
    }

    fn discard(&mut self) -> MachineResult<()> {
        match self.stack.pop() {
            Some(_) => Ok(()),
            None => Err(IllegalStackManipulation),
        }
    }

    fn slide(&mut self, n: uint) -> MachineResult<()> {
        if self.stack.len() < n {
            Err(IllegalStackManipulation)
        } else {
            let top = self.stack.pop().unwrap();
            let mut i = 0u;
            while i < n {
                self.stack.pop();
                i += 1;
            }
            self.stack.push(top);
            Ok(())
        }
    }

    fn calc(&mut self, f: |i64, i64| -> i64) -> MachineResult<()> {
        match self.stack.pop() {
            Some(x) => match self.stack.pop() {
                Some(y) => {
                    self.stack.push(f(x, y));
                    Ok(())
                },
                None => Err(IllegalStackManipulation),
            },
            None => Err(IllegalStackManipulation),
        }
    }

    fn dcalc(&mut self, divf: |i64, i64| -> i64) -> MachineResult<()> {
        match self.stack.pop() {
            Some(x) if x == 0 => Err(ZeroDivision),
            Some(x) => match self.stack.pop() {
                Some(y) => {
                    self.stack.push(divf(x, y));
                    Ok(())
                },
                None => Err(IllegalStackManipulation),
            },
            None => Err(IllegalStackManipulation),
        }
    }

    fn store(&mut self) -> MachineResult<()> {
        match self.stack.pop() {
            Some(val) => match self.stack.pop() {
                Some(addr) => {
                    self.heap.insert(addr, val);
                    Ok(())
                },
                None => Err(IllegalStackManipulation),
            },
            None => Err(IllegalStackManipulation),
        }
    }

    fn retrieve(&mut self) -> MachineResult<()> {
        match self.stack.pop() {
            Some(addr) => {
                self.stack.push(match self.heap.find(&addr) {
                    Some(val) => *val,
                    None => 0,
                });
                Ok(())
            },
            None => Err(IllegalStackManipulation),
        }
    }

    fn call(&mut self, program: &mut ByteCodeReader, index: &mut HashMap<i64, u64>, caller: &mut Vec<u64>, label: &i64) -> MachineResult<()> {
        match program.tell() {
            Ok(pos) => {
                caller.push(pos);
                self.jump(program, index, label)
            },
            Err(err) => Err(MachineIoError(err)),
        }
    }

    fn jump(&mut self, program: &mut ByteCodeReader, index: &mut HashMap<i64, u64>, label: &i64) -> MachineResult<()> {
        match index.find_copy(label) {
            Some(pos) => match program.seek(pos.to_i64().unwrap(), SeekSet) {
                Ok(_) => Ok(()),
                Err(err) => Err(MachineIoError(err)),
            },
            None => {
                loop {
                    match program.read_inst() {
                        Ok((opcode, operand)) if opcode == bc::CMD_MARK => {
                            match program.tell() {
                                Ok(pos) => {
                                    index.insert(operand, pos);
                                    if operand == *label { return Ok(()) }
                                },
                                Err(err) => return Err(MachineIoError(err)),
                            }
                        },
                        Err(ref e) if e.kind == EndOfFile => return Err(UndefinedLabel),
                        Err(err) => return Err(MachineIoError(err)),
                        _ => continue,
                    }
                }
            },
        }
    }

    fn jump_if(&mut self, program: &mut ByteCodeReader, index: &mut HashMap<i64, u64>, label: &i64, test: |i64| -> bool) -> MachineResult<()> {
        match self.stack.pop() {
            Some(x) if test(x) => self.jump(program, index, label),
            None => Err(IllegalStackManipulation),
            _ => Ok(()),
        }
    }

    fn do_return(&mut self, program: &mut ByteCodeReader, caller: &mut Vec<u64>) -> MachineResult<()> {
        match caller.pop() {
            Some(to_return) => match program.seek(to_return.to_i64().unwrap(), SeekSet) {
                Ok(_) => Ok(()),
                Err(err) => Err(MachineIoError(err)),
            },
            None => Err(CallStackEmpty),
        }
    }

    fn put_char(&mut self) -> MachineResult<()> {
        match self.stack.pop() {
            Some(n) if n >= 0 => {
                match write!(self.stdout, "{}", n.to_u8().unwrap() as char) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(MachineIoError(e)),
                }
            },
            Some(_) => Err(IllegalStackManipulation),
            None => Err(IllegalStackManipulation),
        }
    }

    fn put_num(&mut self) -> MachineResult<()> {
        match self.stack.pop() {
            Some(n) => {
                match write!(self.stdout, "{}", n) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(MachineIoError(e)),
                }
            },
            None => Err(IllegalStackManipulation),
        }
    }

    fn get_char(&mut self) -> MachineResult<()> {
        match self.stdin.read_char() {
            Ok(c) => {
                self.stack.push(c as i64);
                try!(self.store());
                Ok(())
            },
            Err(err) => Err(MachineIoError(err)),
        }
    }

    fn get_num(&mut self) -> MachineResult<()> {
        match self.stdin.read_line() {
            Ok(line) => match from_str(line.replace("\n","").as_slice()) {
                Some(n) => {
                    self.stack.push(n);
                    try!(self.store());
                    Ok(())
                },
                None => Err(MachineIoError(standard_error(InvalidInput))),
            },
            Err(err) => Err(MachineIoError(err)),
        }
    }
}

pub trait Interpreter<S> {
    fn run<B: Buffer>(&self, &mut B) -> MachineResult<()>;
}

impl<S: Syntax> Interpreter<S> for S {
    fn run<B: Buffer>(&self, buffer: &mut B) -> MachineResult<()> {
        let mut writer = MemWriter::new();
        match self.compile(buffer, &mut writer) {
            Err(e) => Err(MachineIoError(e)),
            _ => {
                let mut reader = MemReader::new(writer.unwrap());
                let mut machine = Machine::new();
                match machine.run(&mut reader) {
                    Err(e) => Err(e),
                    _ => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::{MemReader, MemWriter};
    use super::*;
    use bytecode::ByteCodeWriter;

    #[test]
    fn test_stack() {
        let mut bcw = MemWriter::new();
        bcw.write_push(1).unwrap();
        bcw.write_dup().unwrap();
        bcw.write_copy(1).unwrap();
        bcw.write_swap().unwrap();
        bcw.write_discard().unwrap();
        bcw.write_slide(1).unwrap();

        let mut bcr = MemReader::new(bcw.unwrap());
        let mut vm = Machine::new();
        let mut caller = vec!();
        let mut index = HashMap::new();
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1, 1));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1, 1, 1));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1, 1, 1));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1, 1));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1));
        assert!(vm.step(&mut bcr, &mut index, &mut caller).is_err());
    }

    #[test]
    fn test_arithmetic() {
        let mut bcw = MemWriter::new();
        bcw.write_add().unwrap();
        bcw.write_sub().unwrap();
        bcw.write_mul().unwrap();
        bcw.write_div().unwrap();
        bcw.write_mod().unwrap();

        let mut bcr = MemReader::new(bcw.unwrap());
        let mut vm = Machine::new();
        let mut caller = vec!();
        let mut index = HashMap::new();
        vm.stack.push_all([2, 19, 2, 5, 1, 1]);
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(2, 19, 2, 5, 2));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(2, 19, 2, 3));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(2, 19, 6));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(2, 3));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(2));
        assert!(vm.step(&mut bcr, &mut index, &mut caller).is_err());
    }

    #[test]
    fn test_heap() {
        let mut bcw = MemWriter::new();
        bcw.write_store().unwrap();
        bcw.write_retrieve().unwrap();

        let mut bcr = MemReader::new(bcw.unwrap());
        let mut vm = Machine::new();
        let mut caller = vec!();
        let mut index = HashMap::new();
        vm.stack.push_all([1, 1, 2]);
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(1));
        assert_eq!(vm.heap.find(&1), Some(&2));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(2));
        assert!(vm.step(&mut bcr, &mut index, &mut caller).is_err());
    }

    #[test]
    fn test_flow() {
        let mut bcw = MemWriter::new();
        bcw.write_jump(1).unwrap();
        bcw.write_mark(3).unwrap();
        bcw.write_call(4).unwrap();
        bcw.write_exit().unwrap();
        bcw.write_mark(2).unwrap();
        bcw.write_jumpn(3).unwrap();
        bcw.write_mark(1).unwrap();
        bcw.write_jumpz(2).unwrap();
        bcw.write_mark(4).unwrap();
        bcw.write_return().unwrap();

        let mut bcr = MemReader::new(bcw.unwrap());
        let mut vm = Machine::new();
        let mut caller = vec!();
        let mut index = HashMap::new();
        vm.stack.push_all([-1, 0]);
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(-1, 0));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!(-1));
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(vm.stack, vec!());
        assert_eq!(caller.len(), 0);
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(caller.len(), 1);
        vm.step(&mut bcr, &mut index, &mut caller).unwrap();
        assert_eq!(caller.len(), 0);
        assert_eq!(vm.step(&mut bcr, &mut index, &mut caller), Ok(false));
    }
}
