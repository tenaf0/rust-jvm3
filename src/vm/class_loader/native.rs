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
        }}, System::registerNatives);

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
            parameters: vec![FieldType::L("java/lang/String".to_string())],
            ret: FieldType::V
        }}, io::println_string);

    native_store
}

mod System {
    use std::sync::atomic::Ordering;
    use smallvec::SmallVec;
    use crate::{ClassRef, VM_HANDLER};
    use crate::vm::class::method::MAX_NO_OF_ARGS;
    use crate::vm::object::ObjectPtr;

    pub fn registerNatives(class: ClassRef, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                           exception: &mut Option<String>) -> Option<u64> {
        let vm = VM_HANDLER.get().unwrap();
        let print_stream = vm.load_class("java/io/PrintStream").unwrap();

        let ptr = vm.object_arena.new_object(print_stream);

        class.data.static_fields[0].store(ptr.ptr as u64, Ordering::Relaxed);

        None
    }
}

mod io {
    use smallvec::SmallVec;
    use crate::{ClassRef, VM_HANDLER};
    use crate::vm::class::method::MAX_NO_OF_ARGS;
    use crate::vm::object::ObjectPtr;
    use crate::vm::pool::string::StrArena;

    pub fn println_int(class: ClassRef, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                   exception: &mut Option<String>) -> Option<u64> {
        println!("{}", args[1] as i32);
        None
    }

    pub fn println_string(class: ClassRef, args: SmallVec<[u64; MAX_NO_OF_ARGS]>,
                          exception: &mut Option<String>) -> Option<u64> {

        let str = args[1];
        let str = ObjectPtr::from_val(str).unwrap();

        println!("{}", StrArena::get_string(str));

        None
    }
}