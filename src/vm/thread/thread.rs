use std::cmp::max;
use std::ops::{Deref, Not};
use std::pin::Pin;
use std::process::exit;
use std::sync::atomic::Ordering;
use smallvec::{SmallVec, smallvec};
use crate::Class;
use crate::vm::class::class::ClassRef;
use crate::vm::class::constant_pool::{CPEntry, SymbolicReference};
use crate::vm::class::method::{Code, MAX_NO_OF_ARGS, Method};
use crate::vm::class_loader::resolve::resolve;
use crate::vm::instructions::{Instruction, instruction_length, InstructionResult};
use crate::vm::thread::frame::Frame;
use crate::vm::thread::thread::ThreadStatus::{FAILED, FINISHED, RUNNING};

pub type MethodRef = (ClassRef, usize);

const STACK_SIZE: usize = 36;

#[derive(Debug)]
pub enum ThreadStatus {
    RUNNING,
    FINISHED(Option<u64>),
    FAILED(String),
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
        self.method(method_ref, &mut frame, arg_no);
        if frame.exception.is_none() {
            self.status = FINISHED(frame.safe_peek());
        } else {
            let string = frame.exception.unwrap();
            self.status = FAILED(string);
        }
    }

    fn method(&mut self, method_ref: MethodRef, prev_frame: &mut Frame, arg_no: usize) {
        let (class, method) = method_ref;
        let class = &*class;
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
                            let res = Self::interpreter_loop(&class, code, frame, &mut result);
                            match res {
                                InstructionResult::Continue => {}
                                InstructionResult::Return => break 'outer
                            }
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

    #[inline]
    fn interpreter_loop(class: &Class, code: &Code, frame: &mut Frame, result: &mut Option<u64>) ->
                                                                               InstructionResult {
        use Instruction::*;

        let instr = code.code[frame.pc];
        let instruction = Instruction::try_from(instr).unwrap();
        println!("{}: {:?}", frame.pc, instruction);
        match instruction {
            iconst_m1 | iconst_0 | iconst_1 | iconst_2 | iconst_3 | iconst_4
            | iconst_5 => {
                let val = instr as isize - 3;
                frame.push(val as u64);
            }
            bipush => {
                let val = code.code[frame.pc + 1];
                frame.push(val as u64);
            }
            sipush => {
                let val = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());
                frame.push(val as u64);
            }
            ldc => {
                let val = code.code[frame.pc + 1];

                let entry = class.get_cp_entry(val as usize);

                match *entry {
                    CPEntry::ConstantString(ptr) => frame.push(ptr.ptr as u64),
                    _ => panic!("")
                }
                println!("Entry: {:?}", entry);
            }
            ldc2_w => {
                let val = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                let entry = class.get_cp_entry(val as usize);

                match *entry {
                    CPEntry::ConstantValue(val) => frame.push(val),
                    _ => panic!("")
                }
                println!("Entry: {:?}", entry);
            }
            istore => {
                let val = frame.pop() as u32;
                let index = code.code[frame.pc + 1];
                frame.set_s(index as usize, val);
            }
            istore_0 | istore_1 | istore_2 | istore_3 => {
                let val = frame.pop() as u32;
                frame.set_s(instr as usize - 59, val);
            }
            iload_0 | iload_1 | iload_2 | iload_3 => {
                let val = frame.get_s(instr as usize - 26);
                frame.push(val as u64);
            }
            iadd => {
                let b = frame.pop() as u32;
                let a = frame.pop() as u32;

                let (res, _) = a.overflowing_add(b);
                frame.push(res as u64);
            }
            isub => {
                let b = frame.pop() as u32;
                let a = frame.pop() as u32;

                let (res, _) = a.overflowing_sub(b);
                frame.push(res as u64);
            }
            imul => {
                let b = frame.pop() as u32;
                let a = frame.pop() as u32;

                let (res, _) = a.overflowing_mul(b);
                frame.push(res as u64);
            }
            iinc => {
                let index = code.code[frame.pc + 1] as usize;
                let cons = code.code[frame.pc + 2] as i8;

                let num = frame.get_s(index) as i32;
                frame.set_s(index, (num + cons as i32) as u32);
            }
            i2l => {}
            if_icmpge | if_icmpgt => {
                let offset = i16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap()) as isize;
                let b = frame.pop() as i32;
                let a = frame.pop() as i32;

                let cmp = match instruction {
                    if_icmpge => a >= b,
                    if_icmpgt => a > b,
                    _ => panic!()
                };
                if cmp {
                    frame.pc = (frame.pc as isize + offset) as usize;
                } else {
                    frame.pc += instruction_length(instruction);
                }
            }
            goto => {
                let offset = i16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap()) as isize;

                frame.pc = (frame.pc as isize + offset) as usize;
            }
            lreturn => {
                *result = Some(frame.pop());
                return InstructionResult::Return;
            }
            _return => {
                return InstructionResult::Return;
            }
            getstatic => {
                use SymbolicReference::FieldReference;

                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match *entry {
                    CPEntry::ResolvedSymbolicReference(FieldReference(class, false, index))
                    => {
                        frame.push(class.data.static_fields[index].load(Ordering::Relaxed));
                    }
                    _ => panic!("Unexpected pattern: {:?}", entry)
                }
            }
            putstatic => {
                use SymbolicReference::FieldReference;

                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match *entry {
                    CPEntry::ResolvedSymbolicReference(FieldReference(class, false, index))
                    => {
                        class.data.static_fields[index].store(frame.pop(), Ordering::Relaxed);
                    }
                    _ => panic!("Unexpected pattern: {:?}", entry)
                }
            }

            _ => todo!()
        }
        println!("{:?}", frame);

        if [if_icmpge, if_icmpgt, goto].contains(&instruction).not() {
            frame.pc += instruction_length(instruction);
        }

        InstructionResult::Continue
    }
}