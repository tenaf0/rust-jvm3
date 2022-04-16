use std::cmp::max;
use std::fmt::{Debug};
use std::sync::atomic::{AtomicU64, Ordering};
use smallvec::{SmallVec};
use crate::{Class, Method, VM_HANDLER};
use crate::class_parser::constants::{AccessFlagMethod};
use crate::helper::{ftou, has_flag, utof};
use crate::vm::class::class::ClassRef;
use crate::vm::class::constant_pool::{CPEntry, SymbolicReference};
use crate::vm::class::constant_pool::SymbolicReference::{FieldReference, MethodReference};
use crate::vm::class::field::FieldType;
use crate::vm::class::method::{Code, MAX_NO_OF_ARGS, MethodRepr};
use crate::vm::class_loader::resolve::resolve;
use crate::vm::instructions::{Instruction, instruction_length, InstructionResult};
use crate::vm::object::ObjectPtr;
use crate::vm::thread::frame::Frame;
use crate::vm::thread::thread::ThreadStatus::{FAILED, FINISHED, RUNNING};
use num_enum::FromPrimitive;

pub type MethodRef = (ClassRef, usize);

const STACK_SIZE: usize = 36;

const PRINT_TRACE: bool = false;
pub const ENABLE_STATS: bool = true;

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

        self.stack.push(frame);

        self.status = RUNNING;
        self.method(method_ref, arg_no);

        let frame = self.stack.last().unwrap();

        if frame.exception.is_none() {
            self.status = FINISHED(frame.safe_peek());
        } else {
            let string = frame.exception.as_ref().unwrap();
            self.status = FAILED(string.clone());
        }
    }

    fn method(&mut self, method_ref: MethodRef, arg_no: usize) {
        let (class, method) = method_ref;
        let class = &*class;
        let method = &class.data.methods[method];

        if PRINT_TRACE {
            println!("{}:{} {:?}", class.data.name, method.name, method.descriptor);
        }

        match &method.repr {
            MethodRepr::Jvm(jvm_method) => {
                if let Some(code) = &jvm_method.code {
                    let mut frame = Frame::new(code.max_locals, code.max_stack);

                    let args = self.stack.last_mut().unwrap().pop_args(arg_no);
                    let mut index = 0;
                    let mut i = 0;
                    if !method.is_static() {
                        frame.set_d(index, args[i]);
                        index += 1;
                        i += 1;
                    }
                    for p in method.descriptor.parameters.iter() {
                        match p {
                            FieldType::D | FieldType::J => {
                                frame.set_d(index, args[i]);
                                index += 2;
                            }
                            _ => {
                                frame.set_s(index, args[i] as u32);
                                index += 1;
                            }
                        }
                        i += 1;
                    }

                    self.stack.push(frame);

                    let mut result = None;
                    'outer: loop {
                        loop {
                            let res = self.interpreter_loop(&class, code, &mut result);
                            match res {
                                InstructionResult::Continue => {}
                                InstructionResult::Return => break 'outer
                            }
                        }

                        // TODO: Exception handling
                    }

                    match result {
                        Some(res) => {
                            self.stack.pop();
                            self.stack.last_mut().unwrap().push(res);
                        },
                        None if method.descriptor.ret != FieldType::V =>
                            panic!("Method should return a value!"),
                        _ => { self.stack.pop(); }
                    }
                } else {
                    panic!("Executing method without code!");
                }
            }
            MethodRepr::Native(native_method) => {
                let fn_ptr = native_method.fn_ptr;
                let prev_frame = self.stack.last_mut().unwrap();
                let args = prev_frame.pop_args(arg_no);
                let exception = &mut prev_frame.exception;
                let res = fn_ptr(ClassRef::new(class), args, exception);

                match (res, exception) {
                    (_, Some(e)) => panic!("Exception occured: {:?}", e),
                    (Some(res), _)  => prev_frame.push(res),
                    (None, _) if method.descriptor.ret != FieldType::V =>
                        panic!("Method {:?} should return a value!", method),
                    _ => {}
                }
            }
        }
    }

    #[inline]
    fn interpreter_loop(&mut self, class: &Class, code: &Code, result: &mut Option<u64>) ->
                                                                               InstructionResult {
        use Instruction::*;

        let frame = self.stack.last_mut().unwrap();
        let instr = code.code[frame.pc];
        let instruction = Instruction::from_primitive(instr);

        if PRINT_TRACE {
            println!("{}: {:?}", frame.pc, instruction);
        }

        if ENABLE_STATS {
            let vm = VM_HANDLER.get().unwrap();
            vm.last_instruction.store(instr, Ordering::Release);
        }

        match instruction {
            nop => {},
            aconst_null => frame.push(0),
            iconst_m1 | iconst_0 | iconst_1 | iconst_2 | iconst_3 | iconst_4
            | iconst_5 => {
                let val = instr as isize - 3;
                frame.push(val as u64);
            }
            lconst_0 | lconst_1 => {
                let val = instr as u64 - 9;
                frame.push(val);
            }
            dconst_0 => {
                const VAL: f64 = 0.0;
                frame.push(ftou(VAL));
            }
            dconst_1 => {
                const VAL: f64 = 1.0;
                frame.push(ftou(VAL));
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
                    CPEntry::ConstantValue(val) => frame.push((val as u32) as u64),
                    // TODO: Reference to class/method can also be loaded
                    _ => panic!("Unexpected entry {:?}", entry)
                }
            }
            ldc2_w => {
                let val = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                let entry = class.get_cp_entry(val as usize);

                match *entry {
                    CPEntry::ConstantValue(val) => frame.push(val),
                    _ => panic!("")
                }
            }
            aaload => {
                let index = frame.pop() as usize;
                let array = frame.pop();
                let array = ObjectPtr::from_val(array).unwrap();

                frame.push(array.get_from_array(index));
            }
            istore => {
                let val = frame.pop() as u32;
                let index = code.code[frame.pc + 1];
                frame.set_s(index as usize, val);
            }
            dstore => {
                let val = frame.pop();
                let index = code.code[frame.pc + 1];
                frame.set_d(index as usize, val);
            }
            astore => {
                let val = frame.pop();
                let index = code.code[frame.pc + 1];
                frame.set_d(index as usize, val);
            }
            istore_0 | istore_1 | istore_2 | istore_3 => {
                let val = frame.pop() as u32;
                frame.set_s(instr as usize - 59, val);
            }
            iload => {
                let index = code.code[frame.pc + 1];
                let val = frame.get_s(index as usize);
                frame.push(val as u64);
            }
            dload => {
                let index = code.code[frame.pc + 1];
                let val = frame.get_d(index as usize);
                frame.push(val);
            }
            aload => {
                let index = code.code[frame.pc + 1];
                let val = frame.get_d(index as usize);
                frame.push(val);
            }
            iload_0 | iload_1 | iload_2 | iload_3 => {
                let val = frame.get_s(instr as usize - 26);
                frame.push(val as u64);
            }
            lload_0 | lload_1 | lload_2 => {
                let val = frame.get_d(instr as usize - 30);
                frame.push(val);
            }
            dload_0 | dload_1 | dload_2 | dload_3 => {
                let val = frame.get_d(instr as usize - 38);
                frame.push(val);
            }
            aload_0 | aload_1 | aload_2 | aload_3 => {
                let objectref = frame.get_d(instr as usize - 42);

                frame.push(objectref);
            }
            iaload => {
                let index = frame.pop();
                let obj = frame.pop();
                let obj = ObjectPtr::from_val(obj).unwrap();

                frame.push(obj.get_from_array(index as usize));
            }
            lstore_0 | lstore_1 | lstore_2  => {
                let val = frame.pop();

                frame.set_d(instr as usize - 63, val);
            }
            dstore_0 | dstore_1 | dstore_2 | dstore_3 => {
                let val = frame.pop();

                frame.set_d(instr as usize - 71, val);
            }
            astore_0 | astore_1 | astore_2 | astore_3 => {
                let objectref = frame.pop();

                frame.set_d(instr as usize - 75, objectref);
            }
            iastore => {
                let val = frame.pop() as u32;
                let index = frame.pop();
                let obj = frame.pop();
                let obj = ObjectPtr::from_val(obj).unwrap();

                obj.store_to_array(index as usize, val as u64);
            }
            aastore => {
                let val = frame.pop();
                let index = frame.pop();
                let obj = frame.pop();
                let obj = ObjectPtr::from_val(obj).unwrap();

                obj.store_to_array(index as usize, val);
            }
            pop => {
                let _ = frame.pop();
            }
            dup => {
                let val = frame.safe_peek().unwrap();
                frame.push(val);
            }
            iadd => {
                let b = frame.pop() as i32;
                let a = frame.pop() as i32;

                let (res, _) = a.overflowing_add(b);
                frame.push(res as u64);
            }
            ladd => {
                let b = frame.pop() as i64;
                let a = frame.pop() as i64;

                let (res, _) = a.overflowing_add(b);
                frame.push(res as u64);
            }
            dadd => {
                let b = utof(frame.pop());
                let a = utof(frame.pop());

                let res = a + b;
                frame.push(ftou(res));
            }
            isub => {
                let b = frame.pop() as u32;
                let a = frame.pop() as u32;

                let (res, _) = a.overflowing_sub(b);
                frame.push(res as u64);
            }
            dsub => {
                let b = utof(frame.pop());
                let a = utof(frame.pop());

                let res = a - b;
                frame.push(ftou(res));
            }
            imul => {
                let b = frame.pop() as u32;
                let a = frame.pop() as u32;

                let (res, _) = a.overflowing_mul(b);
                frame.push(res as u64);
            }
            dmul => {
                let b = utof(frame.pop());
                let a = utof(frame.pop());

                let res = a * b;
                frame.push(ftou(res));
            }
            ddiv => {
                let b = utof(frame.pop());
                let a = utof(frame.pop());

                let res = a / b;
                frame.push(ftou(res));
            }
            dneg => {
                let a = utof(frame.pop());
                frame.push(ftou(-a));
            }
            iinc => {
                let index = code.code[frame.pc + 1] as usize;
                let cons = code.code[frame.pc + 2] as i8;

                let num = frame.get_s(index) as i32;
                frame.set_s(index, (num + cons as i32) as u32);
            }
            i2l => {}
            l2i => {
                let val = frame.pop() as u32;

                frame.push(val as u64)
            }
            ifne => {
                let offset = i16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap()) as isize;

                let a = frame.pop() as i32;
                let cmp = match instruction {
                    ifne => a != 0,
                    _ => panic!()
                };
                if cmp {
                    frame.pc = (frame.pc as isize + offset) as usize;
                } else {
                    frame.pc += instruction_length(instruction);
                }
            }
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
            ireturn => {
                *result = Some(frame.pop());
                return InstructionResult::Return;
            }
            lreturn | dreturn | areturn => {
                *result = Some(frame.pop());
                return InstructionResult::Return;
            }
            _return => {
                return InstructionResult::Return;
            }
            getstatic => {
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
            getfield => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match entry {
                    CPEntry::ResolvedSymbolicReference(FieldReference(_class, true, index)) => {
                        let obj = frame.pop();
                        let obj = ObjectPtr { ptr: obj as *const AtomicU64 };

                        frame.push(obj.get_field(*index));
                    }
                    _ => panic!("Unexpected pattern: {:?}", entry)
                }
            }
            putfield => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                let value = frame.pop();
                let obj = frame.pop();

                let obj = ObjectPtr::from_val(obj).unwrap();

                match entry {
                    CPEntry::ResolvedSymbolicReference(FieldReference(_class, true, index)) => {
                        obj.put_field(*index, value);
                    }
                    _ => panic!("Unexpected pattern: {:?}", entry)
                }
            }
            invokevirtual => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match entry {
                    CPEntry::ResolvedSymbolicReference(MethodReference(other_class, index)) => {
                        let method = &other_class.data.methods[*index];

                        let arg_no = method.descriptor.parameters.len();

                        let obj = frame.peek_nth(arg_no);
                        let obj = ObjectPtr::from_val(obj).unwrap(); // TODO: NPE
                        let obj_class = obj.get_class();

                        let res = invoke_virtual(obj_class, *other_class, (*other_class, *index))
                            .unwrap();

                        self.method(res, arg_no + 1);
                    }
                    _ => panic!()
                }
            }
            invokespecial => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match entry {
                    CPEntry::ResolvedSymbolicReference(MethodReference(other_class, index)) => {
                        // TODO: other_class may differ if direct superclass
                        let method = &other_class.data.methods[*index];

                        let res = invoke_special(*other_class, method).unwrap();

                        self.method(res, 1);
                    }
                    _ => panic!()
                }
            }
            invokestatic => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match entry {
                    CPEntry::ResolvedSymbolicReference(MethodReference(other_class, index)) => {
                        let method = &other_class.data.methods[*index];
                        assert!(method.is_static());

                        self.method((*other_class, *index), method.descriptor.parameters.len());
                    }
                    _ => panic!()
                }
            }
            new => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match *entry {
                    CPEntry::ResolvedSymbolicReference(
                        SymbolicReference::ClassReference(other_class)) => {
                        // TODO: It should not be an abstract class
                        let object = VM_HANDLER.get().unwrap().object_arena.new_object(other_class);
                        frame.push(object.ptr as u64);
                    }
                    _ => panic!("Unexpected pattern: {:?}", entry)
                }
            }
            newarray => {
                let atype = code.code[frame.pc + 1];
                let name = FieldType::convert_newarray_type(atype);
                let length = frame.pop();

                let vm = VM_HANDLER.get().unwrap();
                let array_class = vm.load_class(name).unwrap();

                let object = vm.object_arena.new_array(array_class, length as usize);
                frame.push(object.ptr as u64);
            }
            anewarray => {
                let index = u16::from_be_bytes(code.code[frame.pc + 1..frame.pc + 3].try_into()
                    .unwrap());
                let length = frame.pop();

                let vm = VM_HANDLER.get().unwrap();

                resolve(ClassRef::new(class), index as usize);
                let entry = class.get_cp_entry(index as usize);

                match *entry {
                    CPEntry::ResolvedSymbolicReference(
                        SymbolicReference::ClassReference(other_class)) => {
                        let mut array_class = other_class.data.name.clone();
                        array_class.insert(0, '[');
                        let array_class = vm.load_class(array_class.as_str()).unwrap();

                        let object = vm.object_arena.new_array(array_class, length as usize);
                        frame.push(object.ptr as u64);
                    }
                    _ => panic!("Unexpected pattern: {:?}", entry)
                }
            },
            arraylength => {
                let obj = frame.pop();
                let obj = ObjectPtr::from_val(obj).unwrap();
                frame.push(obj.get_field(0));
            }
            breakpoint | impdep1 | impdep2 => todo!("Instruction {} not yet implemented", instr),
        }

        if PRINT_TRACE {
            for i in 0..self.stack.len() {
                println!("{:?}", self.stack[i]);
            }
        }

        if instr < 153 || instr > 167 {
            let frame = self.stack.last_mut().unwrap();
            frame.pc += instruction_length(instruction);
        }

        InstructionResult::Continue
    }
}

fn invoke_special(class: ClassRef, method: &Method) -> Result<MethodRef, String> {
    // TODO: other_class may differ if direct superclass

    let vm = VM_HANDLER.get().unwrap();

    if let Some(res) = class.find_method(method.name.as_str(), &method.descriptor) {
        return Ok(res)
    } else if !class.is_interface() &&
        !class.data.superclass.ptr().is_null() {
        return invoke_special(class.data.superclass, method)
    }

    if class.is_interface() {
        if let Some(res) = vm.object_class.find_method(method.name.as_str(),
                                                       &method.descriptor) {
            if has_flag(res.0.data.methods[res.1].flag, AccessFlagMethod::ACC_PUBLIC) {
                return Ok(res);
            }
        }
    }

    // TODO: Maximally specific non-abstract interface-method

    Err(format!("Could not resolve method"))
}

fn invoke_virtual(class: ClassRef, resolved_class: ClassRef, method_ref: MethodRef) ->
                                                                                Result<MethodRef, String> {
    let method = &method_ref.0.data.methods[method_ref.1];
    if method.is_private() {
        return Ok(method_ref);
    }

    let search = class.data.methods.iter().enumerate()
        .find(|(i, m)| m.can_override(class, method, resolved_class));
    if let Some((i, m)) = search {
        return Ok((class, i));
    } else {
        return invoke_virtual(class.data.superclass, resolved_class, method_ref);
    }

    // TODO: Maximally specific superinterface
}