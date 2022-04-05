use std::cmp::max;
use std::pin::Pin;
use std::process::exit;
use smallvec::{SmallVec, smallvec};
use crate::Class;
use crate::vm::class::class::ClassRef;
use crate::vm::class::method::{MAX_NO_OF_ARGS, Method};
use crate::vm::instructions::Instruction;
use crate::vm::thread::frame::Frame;
use crate::vm::thread::thread::ThreadStatus::{FAILED, FINISHED, RUNNING};

pub type MethodRef = (ClassRef, usize);

const STACK_SIZE: usize = 36;

pub enum ThreadStatus {
    RUNNING,
    FINISHED(Option<u64>),
    FAILED(String)
}

pub struct VMThread {
    pub status: ThreadStatus,
    stack: SmallVec<[Frame; STACK_SIZE]>
}

impl VMThread {
    pub fn new() -> VMThread {
        VMThread {
            status: FINISHED(None),
            stack: Default::default()
        }
    }

    pub fn start(&mut self, method_ref: MethodRef, args: SmallVec<[u64; MAX_NO_OF_ARGS]>) {
        let arg_no = args.len();
        let mut frame = Frame::new(0, max(arg_no, 1));
        for arg in args {
            frame.push(arg);
        }

        self.status = RUNNING;
        println!("Thread started method: {:?}", method_ref);
        self.method(method_ref, &mut frame, arg_no);
        if frame.exception.is_none() {
            self.status = FINISHED(frame.safe_peek());

            println!("Thread finished execution with return value: {:?}", frame.safe_peek());
        } else {
            let string = frame.exception.unwrap();
            eprintln!("Thread exited with exception: {}", string.clone());

            self.status = FAILED(string);
        }
    }

    fn method(&mut self, method_ref: MethodRef, prev_frame: &mut Frame, arg_no: usize) {
        let (class, method) = method_ref;
        let class = unsafe { &*class.0 };
        let method = &class.data.methods[method];

        match method {
            Method::Jvm(method) => {
                if let Some(code) = &method.code {
                    let mut frame = Frame::new(code.max_locals, code.max_stack);
                    self.stack.push(frame);
                    let args = prev_frame.pop_args(arg_no);
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
                                iconst_m1 | iconst_0 | iconst_1 | iconst_2 | iconst_3 | iconst_4
                                | iconst_5 => {
                                    let val = instr as isize - 3;
                                    frame.push(val as u64);

                                    frame.pc += 1;
                                }
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
            Method::Native(method) => {
                let fn_ptr = method.fn_ptr;
                let args = prev_frame.pop_args(arg_no);
                let exception = &mut prev_frame.exception;
                let res = fn_ptr(args, exception);

                match res {
                    Some(res) if prev_frame.exception.is_none() => prev_frame.push(res),
                    _ => {}
                }
            }
        }
    }
}