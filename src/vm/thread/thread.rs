use std::pin::Pin;
use std::process::exit;
use smallvec::{SmallVec, smallvec};
use crate::Class;
use crate::vm::instructions::Instruction;
use crate::vm::thread::frame::Frame;
use crate::vm::thread::thread::ThreadStatus::{RUNNING, STOPPED};

const MAX_NO_OF_ARGS: usize = 32;

type NativeFnPtr = fn(SmallVec<[u64; MAX_NO_OF_ARGS]>, exception: &mut Option<String>) ->
                                                                                       Option<u64>;
// TODO: Think about return type, args could be a single pointer to the previous frame's
pub enum MethodRef<'a> {
    JVMMethod(&'a Pin<Box<Class>>, usize),
    NativeMethod(NativeFnPtr)
}

const STACK_SIZE: usize = 128;

pub enum ThreadStatus {
    RUNNING,
    STOPPED
}

pub struct VMThread {
    status: ThreadStatus,
    stack: SmallVec<[Frame; STACK_SIZE]>
}

impl VMThread {
    pub fn new() -> VMThread {
        VMThread {
            status: STOPPED,
            stack: Default::default()
        }
    }

    pub fn start(&mut self, method_ref: MethodRef) {
        let mut frame = Frame::new(0, 1); // TODO: Passed arguments

        self.status = RUNNING;
        self.method(method_ref, &mut frame);
        if frame.exception.is_none() {
            self.status = STOPPED;
        } else {
            panic!("Thread exited with exception")
        }
    }

    fn method(&mut self, method_ref: MethodRef, prev_frame: &mut Frame) {
        match method_ref {
            MethodRef::JVMMethod(class, method) => {
                let method = &class.data.method_info[method];
                if let Some(code) = &method.code {
                    let frame = Frame::new(code.max_locals, code.max_stack);
                    self.stack.push(frame);
                    // TODO: Copy method args

                    let frame = self.stack.last_mut().unwrap();
                    let mut result = None;
                    'outer: loop {
                        loop {
                            // Interpreter loop
                            use Instruction::*;

                            let instr = code.code[frame.pc];
                            let instruction = Instruction::try_from(instr).unwrap();
                            println!("{:?}", instruction);
                            match instruction {
                                bipush => {
                                    let val = code.code[frame.pc + 1];
                                    frame.push(val as u64);

                                    frame.pc += 2;
                                }
                                sipush => {
                                    let val = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                                        .unwrap());
                                    frame.push(val as u64);

                                    frame.pc += 3;
                                }
                                istore => {
                                    let val = frame.pop() as u32;
                                    let index = code.code[frame.pc + 1];
                                    frame.set_s(index as usize, val);

                                    frame.pc += 2;
                                }
                                istore_0 | istore_1 | istore_2 | istore_3 => {
                                    let val = frame.pop() as u32;
                                    frame.set_s(instr as usize - 59, val);

                                    frame.pc += 1;
                                }
                                iload_0 | iload_1 | iload_2 | iload_3 => {
                                    let val = frame.get_s(instr as usize - 26);
                                    frame.push(val as u64);

                                    frame.pc += 1;
                                }
                                iadd => {
                                    let a = frame.pop() as u32;
                                    let b = frame.pop() as u32;

                                    let (res, _) = a.overflowing_add(b);
                                    frame.push(res as u64);

                                    frame.pc += 1;
                                }
                                _return => {
                                    break 'outer;
                                }
                                getstatic => {
                                    frame.pc += 3
                                }
                                _ => todo!()
                            }
                            println!("{:?}", frame);
                        }

                        // TODO: Exception handling
                    }

                    if let Some(res) = result {
                        prev_frame.push(res);
                    }
                } else {
                    panic!("Executing method without code!");
                }
            },
            MethodRef::NativeMethod(fn_ptr) => {
                let args = smallvec![];
                let res = fn_ptr(args, &mut prev_frame.exception);

                match res {
                    Some(res) if prev_frame.exception.is_none() => prev_frame.push(res),
                    _ => {}
                }
            }
        }
    }
}