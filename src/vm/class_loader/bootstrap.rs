use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::ptr::{null};
use std::sync::atomic::AtomicU64;
use smallvec::{smallvec, SmallVec};
use crate::{Class, ClassRepr, get_cp_info, Instruction, Method, ObjectHeader, VM, VM_HANDLER, VMThread};
use crate::class_parser::constants::{AccessFlagMethod, CPInfo};
use crate::class_parser::parse_class;
use crate::class_parser::types::ParsedClass;
use crate::class_parser::constants::CPTag;
use crate::helper::{ftou2, has_flag};
use crate::vm::class::class::{AtomicClassState, ClassRef, CPEntryWrapper};
use crate::vm::class::class::ClassState::{Ready, Verified};
use crate::vm::class::constant_pool::{CPEntry, UnresolvedReference};
use crate::vm::class::constant_pool::CPEntry::{ConstantString, ConstantValue, UnresolvedSymbolicReference};
use crate::vm::class::field::{Field, FieldType};
use crate::vm::class::method::{Code, ExceptionHandler, JvmMethod, MethodDescriptor, MethodRepr, NativeMethod};
use crate::vm::class::method::MethodRepr::Native;
use crate::vm::class_loader::array::create_primitive_array_class;
use crate::vm::class_loader::native::{init_native_store, NATIVE_FN_STORE, NativeMethodRef};
use crate::vm::instructions::instruction_length;
use crate::vm::object::ObjectPtr;
use crate::vm::pool::string::StrArena;

use crate::vm::thread::thread::{create_throwable, create_throwable_message, ThreadStatus};
use crate::vm::thread::thread::ThreadStatus::FINISHED;

const INITIAL_CLASS_BUFFER_SIZE: usize = 1024;

impl VM {
    fn add_class(&self, class: Class) -> ClassRef {
        let mut classes = self.classes.lock().unwrap();

        let pin = Box::pin(class);
        classes.push(pin);
        classes.last_mut().unwrap().header.class = &**classes.last().unwrap();

        ClassRef::new(&**classes.last_mut().unwrap())
    }

    pub fn load_bootstrap_classes(&mut self) {
        use std::io::Write;

        let zero_ptr = null();

        let object_name = "java/lang/Object".to_string();
        let object_class_data = Class {
            header: ObjectHeader::default(),
            state: AtomicClassState::new(Ready),
            data: ClassRepr {
                name: object_name.clone(),
                flag: 0,
                superclass: ClassRef::new(zero_ptr),
                interfaces: Default::default(),
                constant_pool: vec![],
                fields: vec![],
                methods: vec![
                    Method {
                        flag: 0,
                        name: "<init>".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![], ret: FieldType::V },
                        repr: MethodRepr::Jvm(JvmMethod {
                            code: Some(Code {
                                max_stack: 0,
                                max_locals: 1,
                                code: vec![
                                    177
                                ],
                                exception_handlers: vec![]
                            })
                        })
                    },
                    Method {
                        flag: AccessFlagMethod::ACC_PUBLIC as u16,
                        name: "equals".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![FieldType::L
                            ("java/lang/Object".to_string())], ret: FieldType::Z },
                        repr: MethodRepr::Jvm(JvmMethod {
                            code: Some(Code {
                                max_stack: 2,
                                max_locals: 2,
                                code: vec![
                                    42, // aload_0
                                    43, // aload_1
                                    166, 0, 5, // if_acmpne
                                    4, // iconst_1
                                    172, // ireturn
                                    3, // iconst_0
                                    172 // ireturn
                                ],
                                exception_handlers: vec![]
                            })
                        })
                    },
                    Method {
                        flag: AccessFlagMethod::ACC_PUBLIC as u16,
                        name: "toString".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![],
                            ret: FieldType::L("java/lang/String".to_string()) },
                        repr: MethodRepr::Native(NativeMethod {
                            fn_ptr: |_, args, _| {
                                let this = ObjectPtr::from_val(args[0]).unwrap();
                                let class_name = &this.get_class().data.name;

                                let mut buf = Vec::with_capacity(class_name.len() + 16);
                                let _ = write!(&mut buf, "{}@{}", class_name, args[0]);

                                let ptr = VM_HANDLER.get().unwrap()
                                    .string_pool.add_string(std::str::from_utf8(&buf).unwrap());

                                Some(ptr.to_val())
                        }
                        })
                    }
                ],
                static_fields: Default::default(),
                instance_field_count: 0
            }
        };

        let object_class = self.add_class(object_class_data);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(object_name, object_class);
        }

        let classloader_name = "java/lang/ClassLoader".to_string();
        let classloader_class_data = Class {
            header: ObjectHeader::default(),
            state: AtomicClassState::new(Ready),
            data: ClassRepr {
                name: classloader_name.clone(),
                flag: 0,
                superclass: object_class,
                interfaces: Default::default(),
                constant_pool: vec![],
                fields: vec![],
                methods: vec![
                    Method {
                        flag: 0,
                        name: "loadClass".to_string(),
                        descriptor: MethodDescriptor {
                            parameters: vec![
                                FieldType::L("java/lang/ClassLoader".to_string()),
                                FieldType::L("java/lang/String".to_string())],
                            ret: FieldType::L("java/lang/Class".to_string()) },
                        repr: MethodRepr::Native(NativeMethod { fn_ptr: |thread, args, exc| {
                            // first argument is the bootstrap class loader (null), second is a
                            // String object which denotes the name of the class that should be loaded

                            let string = args[1] as *const AtomicU64;
                            let obj = ObjectPtr { ptr: string };

                            let vm = VM_HANDLER.get().unwrap();
                            let res = vm.load_class(StrArena::get_string(obj).as_str());

                            match res {
                                Ok(val) => Some(val.ptr() as u64),
                                Err(e) => {
                                    *exc = Some(create_throwable_message("java/lang/Exception",
                                                                         thread, &e));
                                    None
                                }
                            }
                        } })
                    }

                ],
                static_fields: Default::default(),
                instance_field_count: 0
            }
        };

        let classloader_class = self.add_class(classloader_class_data);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(classloader_name, classloader_class);
        }

        let string_name = "java/lang/String".to_string();
        let string_class_data = Class {
            header: ObjectHeader::default(),
            state: AtomicClassState::new(Ready),
            data: ClassRepr {
                name: string_name.clone(),
                flag: 0,
                superclass: object_class,
                interfaces: Default::default(),
                constant_pool: vec![
                    CPEntryWrapper::new(&UnresolvedSymbolicReference(
                            UnresolvedReference::ClassReference("java/lang/String".to_string())
                        )),
                    CPEntryWrapper::new(&UnresolvedSymbolicReference(
                            UnresolvedReference::FieldReference(1, "length".to_string(),
                                                                FieldType::J)
                        )),
                    CPEntryWrapper::new(&UnresolvedSymbolicReference(
                        UnresolvedReference::ClassReference("java/lang/StringUtil".to_string())
                    )),
                    CPEntryWrapper::new(&UnresolvedSymbolicReference(
                        UnresolvedReference::MethodReference(3,
                                                             "stringEquals".to_string(),
                                                            MethodDescriptor {
                                                                parameters: vec![
                                                                    FieldType::L("java/lang/String".to_string()),
                                                                    FieldType::L("java/lang/Object".to_string()),
                                                                ],
                                                                ret: FieldType::Z
                                                            }
                        )
                    )),
                ],
                fields: vec![
                    Field {
                        flag: 0,
                        name: "length".to_string(),
                        descriptor: FieldType::J
                    },
                    Field {
                        flag: 0,
                        name: "index".to_string(),
                        descriptor: FieldType::J
                    }
                ],
                methods: vec![
                    Method {
                        flag: AccessFlagMethod::ACC_PUBLIC as u16,
                        name: "concat".to_string(),
                        descriptor: MethodDescriptor {
                            parameters: vec![FieldType::L("java/lang/String".to_string())],
                            ret: FieldType::L("java/lang/String".to_string()) },
                        repr: Native(NativeMethod {
                            fn_ptr: |thread, args, exc| {
                                let a = match ObjectPtr::from_val(args[0]) {
                                    None => {
                                        *exc = Some(
                                            create_throwable("java/lang/NullPointerException",
                                                           thread));
                                        return None;
                                    }
                                    Some(val) => val
                                };
                                let b = match ObjectPtr::from_val(args[1]) {
                                    None => {
                                        *exc = Some(
                                            create_throwable("java/lang/NullPointerException",
                                                             thread));
                                        return None;
                                    }
                                    Some(val) => val
                                };

                                let b_length = b.get_field(0);
                                if b_length == 0 {
                                    return Some(a.to_val());
                                }

                                let vm = VM_HANDLER.get().unwrap();
                                let res = StrArena::get_string(a) + &StrArena::get_string(b);
                                let res = vm.string_pool.add_string(&res);
                                Some(res.to_val())
                            }
                        })
                    },
                    Method {
                        flag: AccessFlagMethod::ACC_PUBLIC as u16,
                        name: "length".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![], ret: FieldType::J },
                        repr: MethodRepr::Jvm(JvmMethod { code: Some(Code {
                            max_stack: 1,
                            max_locals: 1,
                            code: vec![
                                42, // aload_0
                                180, 0, 2, // getfield #2
                                173 // lreturn
                            ],
                            exception_handlers: vec![]
                        }) })
                    },
                    Method {
                        flag: AccessFlagMethod::ACC_PUBLIC as u16,
                        name: "charAt".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![FieldType::I],
                            ret: FieldType::C },
                        repr: Native(NativeMethod {
                            fn_ptr: |thread, args, exc| {
                                let string = ObjectPtr::from_val(args[0]).unwrap();
                                let index = args[1] as i32;

                                let str = StrArena::get_string(string);

                                match str.encode_utf16().nth(index as usize) {
                                    None => {
                                        *exc = Some(
                                            create_throwable("java/lang/ArrayIndexOutOfBoundsException", thread));

                                        None
                                    }
                                    Some(val) => Some(val as u64)
                                }
                            }
                        })
                    },
                    Method {
                        flag: AccessFlagMethod::ACC_PUBLIC as u16,
                        name: "equals".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![FieldType::L
                            ("java/lang/Object".to_string())], ret: FieldType::Z },
                        repr: MethodRepr::Jvm(JvmMethod { code: Some(Code {
                            max_stack: 2,
                            max_locals: 2,
                            code: vec![
                                42, // aload_0
                                43, // aload_1
                                184, 0, 4, // invokestatic #2
                                172 // ireturn
                            ],
                            exception_handlers: vec![]
                        }) })
                    }
                ],
                static_fields: Default::default(),
                instance_field_count: 2
            }
        };

        let string_class = self.add_class(string_class_data);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(string_name,
                              string_class);
        }

        {
            use FieldType::*;

            for t in [B, C, F, D, Z, S, I, J] {
                let class_data = create_primitive_array_class(t).unwrap();
                let class = self.add_class(class_data);

                {
                    let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
                    class_list.insert(class.data.name.clone(), class);
                }
            }
        }
    }

    pub fn find_loaded_class(&self, name: &str) -> Option<ClassRef> {   // TODO: To support
                                                                        // user-defined class loaders as well,
                                                                        // it should take a class_loader object as well
        self.bootstrap_cl_class_list.lock().unwrap().get(name).map(|s| *s)
    }

    pub fn load_class(&self, name: &str) -> Result<ClassRef, Exception> {
        if let Some(class) = self.find_loaded_class(name) {
            return Ok(class);
        }

        eprintln!("Started loading class: {}", name);

        if name.starts_with("[") {
            return self.load_array_class(name);
        }

        let classpath = self.args.read().unwrap();
        let classpath = classpath.classpath.as_ref().map(|s| s.as_str())
            .unwrap_or(".");
        let mut file = find_class_file(name, classpath)?;
        let mut buf = Vec::with_capacity(INITIAL_CLASS_BUFFER_SIZE);
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

        eprintln!("Loaded class file {:?}", name);

        let class = self.derive_class(ObjectPtr::null(), name, &mut buf)?;

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(class.data.name.clone(),
                              class);
        }

        Ok(class)
    }

    fn load_array_class(&self, name: &str) -> Result<ClassRef, Exception> {
        let component_class = self.load_class(&name[1..])?;

        let class = Class {
            header: Default::default(),
            state: AtomicClassState::new(Verified),
            data: ClassRepr {
                name: name.to_string(),
                flag: component_class.data.flag,
                superclass: ClassRef::new(null()),
                interfaces: Default::default(),
                constant_pool: vec![],
                fields: vec![],
                methods: vec![],
                static_fields: Default::default(),
                instance_field_count: 0
            }
        };


        let class = self.add_class(class);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(class.data.name.clone(), class);
        }

        Ok(class)
    }

    fn load_cp_entries(parsed_class: &ParsedClass, constant_pool: &mut Vec<CPEntry>) -> Result<()
        , Exception> {
        use CPInfo::*;

        for entry in &parsed_class.constant_pool {
            match *entry {
                Integer(v) | Float(v) => constant_pool.push(CPEntry::ConstantValue(v as u64)),
                Long(num1, num2) => constant_pool.push(
                    CPEntry::ConstantValue(
                        (num1 as u64) << 32 | (num2 as u64)
                    )),
                Double(num1, num2) => {
                    let num1 = num1.to_be_bytes();
                    let num2 = num2.to_be_bytes();
                    let mut buf = [0u8; 8];

                    buf[..4].clone_from_slice(&num1[..]);
                    buf[4..].clone_from_slice(&num2[..]);

                    constant_pool.push(ConstantValue(ftou2(f64::from_be_bytes(buf))));
                }

                Class(name) => {
                    let name = get_cp_info!(parsed_class, name, CPTag::Utf8, CPInfo::Utf8(str),
                        str)?.clone();

                    constant_pool.push(UnresolvedSymbolicReference
                        (UnresolvedReference::ClassReference(name)));
                }
                CPInfo::Methodref(class, name_and_type) => {
                    let (name_ind, descriptor_ind) = get_cp_info!(parsed_class, name_and_type,
                        CPTag::NameAndType, CPInfo::NameAndType(name_index, descriptor_index),
                        (*name_index, *descriptor_index))?;

                    let name = get_cp_info!(parsed_class, name_ind, CPTag::Utf8,
                        CPInfo::Utf8(name), name)?.clone();
                    let descriptor = get_cp_info!(parsed_class, descriptor_ind, CPTag::Utf8,
                        CPInfo::Utf8(descriptor), descriptor)?.clone();

                    let descriptor = MethodDescriptor::parse(&descriptor)
                        .ok_or(format!("Could not parse method descriptor {}", descriptor))?;

                    constant_pool.push(UnresolvedSymbolicReference
                        (UnresolvedReference::MethodReference(class, name, descriptor)));
                }
                CPInfo::Fieldref(class, name_and_type) => {
                    let (name_ind, descriptor_ind) = get_cp_info!(parsed_class, name_and_type,
                        CPTag::NameAndType, CPInfo::NameAndType(name_index, descriptor_index),
                        (*name_index, *descriptor_index))?;

                    let name = get_cp_info!(parsed_class, name_ind, CPTag::Utf8,
                        CPInfo::Utf8(name), name)?.clone();
                    let descriptor = get_cp_info!(parsed_class, descriptor_ind, CPTag::Utf8,
                        CPInfo::Utf8(descriptor), descriptor)?.clone();

                    let descriptor = FieldType::parse(descriptor.as_str())
                        .ok_or(format!("Could not parse field descriptor {}", descriptor))?;

                    constant_pool.push(UnresolvedSymbolicReference
                        (UnresolvedReference::FieldReference(class, name, descriptor)));
                }
                CPInfo::InterfaceMethodref(class, name_and_type) => {
                    let (name_ind, descriptor_ind) = get_cp_info!(parsed_class, name_and_type,
                        CPTag::NameAndType, CPInfo::NameAndType(name_index, descriptor_index),
                        (*name_index, *descriptor_index))?;

                    let name = get_cp_info!(parsed_class, name_ind, CPTag::Utf8,
                        CPInfo::Utf8(name), name)?.clone();
                    let descriptor = get_cp_info!(parsed_class, descriptor_ind, CPTag::Utf8,
                        CPInfo::Utf8(descriptor), descriptor)?.clone();

                    let descriptor = MethodDescriptor::parse(&descriptor)
                        .ok_or(format!("Could not parse interface method descriptor {}",
                                       descriptor))?;

                    constant_pool.push(UnresolvedSymbolicReference(
                        UnresolvedReference::InterfaceMethodReference(class, name, descriptor)));
                }

                CPInfo::String(ind) => {
                    let string = get_cp_info!(parsed_class, ind, CPTag::Utf8,
                        CPInfo::Utf8(ind), ind)?.clone();

                    let ptr = VM_HANDLER.get().unwrap().string_pool.intern_string(string.as_str());

                    constant_pool.push(ConstantString(ptr));
                }
                _ => constant_pool.push(CPEntry::Hole)
            }
        }

        Ok(())
    }

    fn load_methods(parsed_class: &ParsedClass, class_name: &str, methods: &mut Vec<Method>) ->
                                                                                         Result<()
        , Exception> {
        for m in &parsed_class.methods {
            let name = get_cp_info!(parsed_class, m.name_index, CPTag::Utf8, CPInfo::Utf8(str),
                        str)?.clone();
            let descriptor = get_cp_info!(parsed_class, m.descriptor_index, CPTag::Utf8, CPInfo::Utf8(str),
                        str)?;
            let descriptor = MethodDescriptor::parse(descriptor)
                .ok_or(format!("Could not parse method descriptor {}", descriptor))?;

            let mut code = None;
            for a in &m.attributes {
                let attribute_type = get_cp_info!(parsed_class, a.attribute_name_index, CPTag::Utf8, CPInfo::Utf8(str),
                        str)?;
                if attribute_type == "Code" {
                    let code_length = u32::from_be_bytes(a.info[4..8].try_into().unwrap()) as usize;
                    let mut code_buf = vec![0; code_length];
                    code_buf.clone_from_slice(&a.info[8..8+code_length]);

                    let mut pc = 0;
                    loop {
                        if code_buf.len() < pc {
                            panic!("Couldn't validate used instrucitons");
                        } else if code_buf.len() == pc {
                            break;
                        }

                        let instr = code_buf[pc];
                        if Instruction::exists(instr) {
                            let instruction = unsafe { Instruction::from_unchecked(instr) };
                            pc += instruction_length(instruction);
                        } else {
                            panic!("Instruction {} is not yet implemented", code_buf[pc]);
                        }
                    }

                    let exception_start = 8+code_length;
                    let exception_table_length = u16::from_be_bytes(
                        a.info[exception_start..exception_start+2].try_into().unwrap()) as usize;

                    let mut exception_start = exception_start+2;
                    let mut handler : Vec<(u16, u16, u16, u16)> = Vec::with_capacity(exception_table_length);
                    for _ in 0..exception_table_length {
                        handler.push((
                            u16::from_be_bytes(
                                a.info[exception_start..exception_start+2].try_into().unwrap()),
                            u16::from_be_bytes(
                                a.info[exception_start+2..exception_start+4].try_into().unwrap()),
                            u16::from_be_bytes(
                                a.info[exception_start+4..exception_start+6].try_into().unwrap()),
                            u16::from_be_bytes(
                                a.info[exception_start+6..exception_start+8].try_into().unwrap())
                            ));
                        exception_start += 8;
                    }

                    let exception_handlers = handler.iter().map(|(a,b,c,d)| {
                        let catch_type = if *d == 0 { None } else {
                            let index = get_cp_info!(parsed_class, *d,
                                CPTag::Class, CPInfo::Class(num), *num).expect("Class_info \
                                structure was expected");
                            let exception_name = get_cp_info!(parsed_class, index, CPTag::Utf8,
                                CPInfo::Utf8(str), str).expect("Utf8_info structure was expected");

                            let vm = VM_HANDLER.get().unwrap().load_class(exception_name).unwrap();

                            Some(vm)
                        };

                        ExceptionHandler {
                            start_pc: *a as usize,
                            end_pc: *b as usize,
                            handler_pc: *c as usize,
                            catch_type
                        }
                    }).collect();

                    code = Some(Code {
                        max_stack: u16::from_be_bytes(a.info[..2].try_into().unwrap()) as usize,
                        max_locals: u16::from_be_bytes(a.info[2..4].try_into().unwrap()) as usize,
                        code: code_buf,
                        exception_handlers
                    });
                }
            }

            let repr = if has_flag(m.access_flags, AccessFlagMethod::ACC_NATIVE) {
                let native_store = NATIVE_FN_STORE.get_or_init(|| init_native_store());

                let method_ref = NativeMethodRef {
                    class_name: class_name.to_string(),
                    method_name: name.clone(),
                    descriptor: descriptor.clone()
                };
                match native_store.get(&method_ref) {
                    Some(res) => {
                        Ok(Native(NativeMethod { fn_ptr: *res }))
                    }
                    None => Err(format!("Could not resolve native method {}", name))
                }
            } else {
                Ok(MethodRepr::Jvm(
                    JvmMethod {
                        code
                    }
                ))
            };

            methods.push(Method {
                flag: m.access_flags,
                name,
                descriptor,
                repr: repr?,
            });
        }

        Ok(())
    }

    fn load_fields(parsed_class: &ParsedClass, fields: &mut Vec<Field>) -> Result<(), Exception> {
        for f in &parsed_class.fields {
            let name = get_cp_info!(parsed_class, f.name_index, CPTag::Utf8, CPInfo::Utf8(str),
                        str)?.clone();
            let descriptor = get_cp_info!(parsed_class, f.descriptor_index, CPTag::Utf8,
                CPInfo::Utf8(str), str)?;
            let descriptor = FieldType::parse(descriptor)
                .ok_or(format!("Could not parse field descriptor {}", descriptor))?;

            fields.push(Field {
                flag: f.access_flags,
                name,
                descriptor
            })
        }

        Ok(())
    }

    pub fn derive_class(&self, _class_loader: ObjectPtr, _name: &str, buf: &[u8]) ->
                                                                                 Result<ClassRef, Exception> {
        let parsed_class = parse_class(buf).map_err(|e| e.to_string())?;

        let mut constant_pool = vec![];
        VM::load_cp_entries(&parsed_class, &mut constant_pool)?;
        let constant_pool: Vec<CPEntryWrapper> = constant_pool.iter()
            .map(CPEntryWrapper::new).collect();

        let this_class = get_cp_info!(parsed_class, parsed_class.this_class, CPTag::Class,
            CPInfo::Class(num), *num)?;

        let class_name = get_cp_info!(parsed_class, this_class, CPTag::Utf8, CPInfo::Utf8(str), str)?;

        let superclass = get_cp_info!(parsed_class, parsed_class.super_class, CPTag::Class,
            CPInfo::Class(num), *num)?;

        let superclass_name = get_cp_info!(parsed_class, superclass, CPTag::Utf8, CPInfo::Utf8
            (str), str)?;

        let ptr = self.string_pool.intern_string(superclass_name);

        let mut thread = VMThread::new();
        thread.start((self.classloader, 0), smallvec![0, ptr.ptr as u64]);

        let superclass: ClassRef;

        match thread.status {
            FINISHED(Some(class)) => superclass = ClassRef::new(class as *const Class),
            ThreadStatus::FAILED(e) => return Err(e),
            _ => panic!("Can't happen")
        }

        let mut methods = Vec::with_capacity(parsed_class.methods.len());
        VM::load_methods(&parsed_class, class_name, &mut methods)?;

        let mut fields = Vec::with_capacity(parsed_class.fields.len());
        VM::load_fields(&parsed_class, &mut fields)?;

        let static_field_count = fields.iter().filter(|f| f.is_static()).count();
        let mut static_fields = SmallVec::with_capacity(static_field_count);
        for _ in 0..static_field_count {
            static_fields.push(AtomicU64::new(0));
        }

        let instance_field_count = superclass.data.instance_field_count + fields.len() -
            static_field_count;

        let class = Class {
            header: ObjectHeader::default(),
            state: AtomicClassState::new(Verified), // TODO: Verification before giving this state
            data: ClassRepr {
                name: class_name.clone(),
                flag: parsed_class.access_flags,
                superclass,
                interfaces: Default::default(),
                constant_pool,
                fields,
                methods,
                static_fields,
                instance_field_count
            }
        };

        let class = self.add_class(class);
        Ok(class)
    }
}

type Exception = String;

pub fn find_class_file(name: &str, class_path: &str) -> Result<File, Exception> {
    let buf = PathBuf::from(class_path);
    let name = name.replace('.', "/") + ".class";

    File::open(buf.join(name.clone()).as_path()).map_err(|e| {
        let mut str = e.to_string();
        str.push_str(format!(" while loading {}", name).as_str());
        str
    })
}