use std::collections::HashMap;
use std::pin::Pin;
use std::ptr::null;
use std::sync::Mutex;
use crate::{Class, ClassRepr, ObjectHeader};
use crate::vm::class::class::ClassRef;
use crate::vm::class::field::FieldType;
use crate::vm::class::method::{Code, JvmMethod, Method, MethodDescriptor, NativeMethod};
use crate::vm::pool::object::ObjectArena;
use crate::vm::pool::string::StringPool;

pub struct VM {
    pub classes: Mutex<                     // Responsible for ensuring that only a single class loader can grow it
        Vec<Pin<Box<Class>>>                // Indirection is required due to pinning -- since
      >,                                    // objects will point to the class directly, we
                                            // can't move them around
    // TODO: Allocate in special class area.

    pub bootstrap_cl_class_list: Mutex<HashMap<String, ClassRef>>,
    pub object_arena: ObjectArena,
    pub string_pool: StringPool,

    pub classloader: ClassRef,
    pub string_class: ClassRef
}

impl VM {
    pub fn init() -> VM {
        let mut vm = VM {
            classes: Mutex::new(vec![]),
            bootstrap_cl_class_list: Default::default(),
            object_arena: Default::default(),
            string_pool: Default::default(),

            classloader: ClassRef::new(null()),
            string_class: ClassRef::new(null())
        };

        vm.load_bootstrap_classes();

        vm.classloader = ClassRef::new(&*vm.classes.lock().unwrap()[1]);
        vm.string_class = ClassRef::new(&*vm.classes.lock().unwrap()[2]);

        vm
    }
}