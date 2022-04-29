use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use crate::vm::class::field::FieldType;
use crate::vm::class::method::{MethodDescriptor, NativeFnPtr, NativeMethod};

#[derive(Eq, Hash, PartialEq)]
pub struct NativeMethodRef {
    pub class_name: String,
    pub method_name: String,
    pub descriptor: MethodDescriptor
}

pub static NATIVE_FN_STORE: OnceCell<HashMap<NativeMethodRef, NativeFnPtr>> = OnceCell::new();

pub fn init_native_store() -> HashMap<NativeMethodRef, NativeFnPtr> {
    let mut native_store: HashMap<NativeMethodRef, NativeFnPtr> = Default::default();

    native_store.insert(NativeMethodRef {
        class_name: "java/lang/System".to_string(),
        method_name: "registerNatives".to_string(),
        descriptor: MethodDescriptor {
                                parameters: vec![],
                                ret: FieldType::V
        }}, lang::system::registerNatives);

    native_store.insert(NativeMethodRef {
        class_name: "java/lang/Integer".to_string(),
        method_name: "parseInt".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::L("java/lang/String".to_string())],
            ret: FieldType::I
        }}, lang::parseInt);

    native_store.insert(NativeMethodRef {
        class_name: "java/lang/Integer".to_string(),
        method_name: "toString".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::I],
            ret: FieldType::L("java/lang/String".to_string())
        }}, lang::toString);

    native_store.insert(NativeMethodRef {
        class_name: "java/lang/Math".to_string(),
        method_name: "sqrt".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::D],
            ret: FieldType::D
        }}, lang::math::sqrt);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "print".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::C],
            ret: FieldType::V
        }}, io::print_char);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "print".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::I],
            ret: FieldType::V
        }}, io::print_int);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "print".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::J],
            ret: FieldType::V
        }}, io::print_long);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "print".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::D],
            ret: FieldType::V
        }}, io::print_double);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "print".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::L("java/lang/String".to_string())],
            ret: FieldType::V
        }}, io::print_string);

    native_store
}

mod lang {
    use std::num::ParseIntError;
    use smallvec::SmallVec;
    use crate::{ClassRef, initialize_class, VM_HANDLER, VMThread};
    use crate::vm::class::method::MAX_NO_OF_ARGS;
    use crate::vm::object::ObjectPtr;
    use crate::vm::pool::string::{StrArena, StringPool};
    use crate::vm::thread::thread::{create_throwable, create_throwable_message};

    pub fn parseInt(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                    exception: &mut Option<ObjectPtr>) -> Option<u64> {
        let string = match ObjectPtr::from_val(args[0]) {
            None => {
                *exception = Some(create_throwable("java/lang/NullPointerException", thread));
                return None;
            }
            Some(val) => val
        };

        let string = StrArena::get_string(string);

        match string.parse::<i32>() {
            Ok(val) => Some(val as u64),
            Err(e) => {
                *exception = Some(create_throwable_message("java/lang/Exception", thread, // TODO: NumberFormatException
                                                           &e.to_string()));
                return None;
            }
        }
    }

    pub fn toString(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                    exception: &mut Option<ObjectPtr>) -> Option<u64> {
        use std::io::Write;

        let mut buf: Vec<u8> = Vec::with_capacity(16);
        write!(&mut buf, "{}", args[0] as i32);

        let str = std::str::from_utf8(&buf).unwrap();
        let VM = VM_HANDLER.get().unwrap();
        let string = VM.string_pool.add_string(str);

        Some(string.to_val())
    }

    pub mod system {
        use std::sync::atomic::Ordering;
        use smallvec::SmallVec;
        use crate::{ClassRef, VM_HANDLER, VMThread};
        use crate::vm::class::method::MAX_NO_OF_ARGS;
        use crate::vm::object::ObjectPtr;

        pub fn registerNatives(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                               exception: &mut Option<ObjectPtr>) -> Option<u64> {
            let vm = VM_HANDLER.get().unwrap();
            let print_stream = vm.load_class("java/io/PrintStream").unwrap();

            let ptr = vm.object_arena.new_object(print_stream);

            thread.stack.last().unwrap().methodref.0.data.static_fields[0].store(ptr.ptr as u64,
                                                                      Ordering::Relaxed);

            None
        }
    }

    pub mod math {
        use smallvec::SmallVec;
        use crate::{ClassRef, VMThread};
        use crate::helper::{ftou2, utof2};
        use crate::vm::class::method::MAX_NO_OF_ARGS;
        use crate::vm::object::ObjectPtr;

        pub fn sqrt(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                    exception: &mut Option<ObjectPtr>) -> Option<u64> {
            let a = utof2(args[0]);
            Some(ftou2(a.sqrt()))
        }
    }
}

mod io {
    use smallvec::SmallVec;
    use crate::{ClassRef, VM_HANDLER, VMThread};
    use crate::helper::{ftou2, utof2};
    use crate::vm::class::method::MAX_NO_OF_ARGS;
    use crate::vm::object::ObjectPtr;
    use crate::vm::pool::string::StrArena;

    pub fn print_char(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                     exception: &mut Option<ObjectPtr>) -> Option<u64> {
        print!("{}", args[1] as u8 as char);
        None
    }

    pub fn print_int(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                     exception: &mut Option<ObjectPtr>) -> Option<u64> {
        print!("{}", args[1] as i32);
        None
    }

    pub fn print_long(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                     exception: &mut Option<ObjectPtr>) -> Option<u64> {
        print!("{}", args[1] as i64);
        None
    }

    pub fn print_double(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                        exception: &mut Option<ObjectPtr>) -> Option<u64> {
        use std::io::Write;

        let mut buf: Vec<u8> = Vec::with_capacity(20);
        write!(&mut buf, "{}", utof2(args[1]));
        let str = std::str::from_utf8(&buf).unwrap();
        if let None = str.find(".") {
            print!("{}.0", str);
        } else {
            print!("{}", str);
        }
        None
    }

    pub fn print_string(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                        exception: &mut Option<ObjectPtr>) -> Option<u64> {

        let str = args[1];
        match ObjectPtr::from_val(str) {
            None => print!("null"),
            Some(str) => print!("{}", StrArena::get_string(str))
        }

        None
    }
}