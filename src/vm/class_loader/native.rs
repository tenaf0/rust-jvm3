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
        class_name: "java/lang/Math".to_string(),
        method_name: "sqrt".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::D],
            ret: FieldType::D
        }}, lang::math::sqrt);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::I],
            ret: FieldType::V
        }}, io::println_int);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::D],
            ret: FieldType::V
        }}, io::println_double);

    native_store.insert(NativeMethodRef {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: MethodDescriptor {
            parameters: vec![FieldType::L("java/lang/String".to_string())],
            ret: FieldType::V
        }}, io::println_string);

    native_store
}

mod lang {
    use smallvec::SmallVec;
    use crate::{ClassRef, initialize_class, VM_HANDLER, VMThread};
    use crate::vm::class::method::MAX_NO_OF_ARGS;
    use crate::vm::object::ObjectPtr;

    pub mod system {
        use std::sync::atomic::Ordering;
        use smallvec::SmallVec;
        use crate::{ClassRef, VM_HANDLER, VMThread};
        use crate::vm::class::method::MAX_NO_OF_ARGS;

        pub fn registerNatives(thread: &VMThread, args: SmallVec<[u64;
            MAX_NO_OF_ARGS]>,
                               exception: &mut Option<String>) -> Option<u64> {
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

        pub fn sqrt(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                    exception: &mut Option<String>) -> Option<u64> {
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

    pub fn println_int(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                   exception: &mut Option<String>) -> Option<u64> {
        println!("{}", args[1] as i32);
        None
    }

    pub fn println_double(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                       exception: &mut Option<String>) -> Option<u64> {
        use std::io::Write;

        let mut buf: Vec<u8> = Vec::with_capacity(20);
        write!(&mut buf, "{}", utof2(args[1]));
        let str = std::str::from_utf8(&buf).unwrap();
        if let None = str.find(".") {
            println!("{}.0", str);
        } else {
            println!("{}", str);
        }
        None
    }

    pub fn println_string(thread: &VMThread, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                          exception: &mut Option<String>) -> Option<u64> {

        let str = args[1];
        match ObjectPtr::from_val(str) {
            None => println!("null"),
            Some(str) => println!("{}", StrArena::get_string(str))
        }

        None
    }
}