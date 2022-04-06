use std::cell::UnsafeCell;
use std::fs::File;
use std::io::ErrorKind::ConnectionAborted;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::{Mutex, RwLock};
use std::sync::atomic::AtomicU64;
use smallvec::{smallvec, SmallVec};
use crate::{Class, ClassRepr, get_cp_info, Method, ObjectHeader, VM, VM_HANDLER, VMThread};
use crate::class_parser::constants::CPInfo;
use crate::class_parser::parse_class;
use crate::class_parser::types::ParsedClass;
use crate::class_parser::constants::CPTag;
use crate::vm::class::class::{ClassRef, ClassState};
use crate::vm::class::constant_pool::{CPEntry, UnresolvedReference};
use crate::vm::class::constant_pool::CPEntry::{ConstantString, ConstantValue, UnresolvedSymbolicReference};
use crate::vm::class::field::{Field, FieldType};
use crate::vm::class::method::{Code, JvmMethod, MethodDescriptor, NativeMethod};
use crate::vm::object::ObjectPtr;
use crate::vm::pool::string::StringObject;
use crate::vm::thread::thread::ThreadStatus;
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
        let zero_ptr = null();

        let object_name = "java/lang/Object".to_string();
        let object_class_data = Class {
            header: ObjectHeader { class: zero_ptr },
            state: Mutex::new(ClassState::Ready),
            data: ClassRepr {
                name: object_name.clone(),
                superclass: ClassRef::new(zero_ptr),
                interfaces: Default::default(),
                constant_pool: vec![],
                fields: vec![],
                methods: vec![],
                static_fields: Default::default()
            }
        };

        let object_class = self.add_class(object_class_data);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(object_name, object_class);
        }

        let classloader_name = "java/lang/ClassLoader".to_string();
        let classloader_class_data = Class {
            header: ObjectHeader { class: zero_ptr },
            state: Mutex::new(ClassState::Ready),
            data: ClassRepr {
                name: classloader_name.clone(),
                superclass: object_class,
                interfaces: Default::default(),
                constant_pool: vec![],
                fields: vec![],
                methods: vec![
                    Method::Native(NativeMethod { fn_ptr: |args, exc| {
                        // first argument is the bootstrap class loader (null), second is a
                        // String object which denotes the name of the class that should be loaded

                        let string = args[1] as *mut u64;
                        let string = ObjectPtr { ptr: string };
                        let index = string.get_field(0);

                        let vm = VM_HANDLER.get().unwrap();
                        let string = unsafe { &vm.string_pool.get(index as usize ).read()};
                        let res = vm.load_class(string);

                        match res {
                            Ok(val) => Some(val.ptr() as u64),
                            Err(e) => {
                                *exc = Some(e);
                                None
                            }
                        }
                    } })
                ],
                static_fields: Default::default()
            }
        };

        let classloader_class = self.add_class(classloader_class_data);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(classloader_name, classloader_class);
        }

        let string_name = "java/lang/String".to_string();
        let string_class_data = Class {
            header: ObjectHeader { class: zero_ptr },
            state: Mutex::new(ClassState::Ready),
            data: ClassRepr {
                name: string_name.clone(),
                superclass: object_class,
                interfaces: Default::default(),
                constant_pool: vec![],
                fields: vec![],
                methods: vec![],
                static_fields: Default::default()
            }
        };

        let string_class = self.add_class(string_class_data);

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(string_name,
                              string_class);
        }
    }

    pub fn find_loaded_class(&self, name: &str) -> Option<ClassRef> {   // TODO: To support
                                                                        // user-defined class loaders as well,
                                                                        // it should take a class_loader object as well
        self.bootstrap_cl_class_list.lock().unwrap().get(name).map(|s| *s)
    }

    pub fn load_class(&self, name: &str) -> Result<ClassRef, Exception> {
        println!("Started loading class: {}", name);
        if let Some(class) = self.find_loaded_class(name) {
            return Ok(class);
        }

        let mut file = find_class_file(name, "./jdk/target")?;
        let mut buf = Vec::with_capacity(INITIAL_CLASS_BUFFER_SIZE);
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

        println!("Loaded class file {:?}", name);

        let class = self.derive_class(ObjectPtr::null(), name, &mut buf)?;

        {
            let mut class_list = self.bootstrap_cl_class_list.lock().unwrap();
            class_list.insert(class.data.name.clone(),
                              class);
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

                    constant_pool.push(ConstantValue(f64::from_be_bytes(buf) as u64));
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

                CPInfo::String(ind) => {
                    let string = get_cp_info!(parsed_class, ind, CPTag::Utf8,
                        CPInfo::Utf8(ind), ind)?.clone();

                    let ptr = VM_HANDLER.get().unwrap().string_pool.add_string(string.as_str());
                    // TODO: It should be interned

                    constant_pool.push(ConstantString(ptr));
                }
                _ => constant_pool.push(CPEntry::Hole)
            }
        }

        Ok(())
    }

    fn load_methods(parsed_class: &ParsedClass, methods: &mut Vec<Method>) -> Result<()
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

                    code = Some(Code {
                        max_stack: u16::from_be_bytes(a.info[..2].try_into().unwrap()) as usize,
                        max_locals: u16::from_be_bytes(a.info[2..4].try_into().unwrap()) as usize,
                        code: code_buf
                    });
                }
            }

            methods.push(Method::Jvm(JvmMethod {
                name,
                descriptor,
                code
            }));
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

    pub fn derive_class(&self, class_loader: ObjectPtr, name: &str, buf: &[u8]) ->
                                                                                 Result<ClassRef, Exception> {
        let parsed_class = parse_class(buf).map_err(|e| e.to_string())?;

        let mut constant_pool = vec![];
        VM::load_cp_entries(&parsed_class, &mut constant_pool)?;
        let mut constant_pool: Vec<UnsafeCell<CPEntry>> = constant_pool.iter().map(|e|
            UnsafeCell::new(e.clone())).collect();

        let this_class = get_cp_info!(parsed_class, parsed_class.this_class, CPTag::Class,
            CPInfo::Class(num), *num)?;

        let class_name = get_cp_info!(parsed_class, this_class, CPTag::Utf8, CPInfo::Utf8(str), str)?;

        let superclass = get_cp_info!(parsed_class, parsed_class.super_class, CPTag::Class,
            CPInfo::Class(num), *num)?;

        let superclass_name = get_cp_info!(parsed_class, superclass, CPTag::Utf8, CPInfo::Utf8
            (str), str)?;

        let ptr = self.string_pool.add_string(superclass_name);

        let mut thread = VMThread::new();
        thread.start((self.classloader, 0), smallvec![0, ptr.ptr as u64]);

        let mut superclass: ClassRef;

        match thread.status {
            FINISHED(Some(class)) => superclass = ClassRef::new(class as *const Class),
            ThreadStatus::FAILED(e) => return Err(e),
            _ => panic!("Can't happen")
        }

        // println!("{:#?}", constant_pool);
        println!("{:?}", *superclass);

        let mut methods = Vec::with_capacity(parsed_class.methods.len());
        VM::load_methods(&parsed_class, &mut methods)?;

        let mut fields = Vec::with_capacity(parsed_class.fields.len());
        VM::load_fields(&parsed_class, &mut fields)?;

        let static_field_count = fields.iter().filter(|f| f.is_static()).count();
        let mut static_fields = SmallVec::with_capacity(static_field_count);
        for _ in 0..static_field_count {
            static_fields.push(AtomicU64::new(0));
        }

        let class = Class {
            header: ObjectHeader { class: null() },
            state: Mutex::new(ClassState::Verified), // TODO: Verification before giving this state
            data: ClassRepr {
                name: class_name.clone(),
                superclass,
                interfaces: Default::default(),
                constant_pool,
                fields,
                methods,
                static_fields
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

    File::open(buf.join(name).as_path()).map_err(|e| e.to_string())
}