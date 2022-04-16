use std::collections::HashMap;
use std::fs::File;
use std::pin::Pin;
use std::ptr::null;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use crate::{Class};
use crate::vm::class::class::ClassRef;
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

    pub object_class: ClassRef,
    pub classloader: ClassRef,
    pub string_class: ClassRef,

    pub stat_file: Mutex<File>,
    pub last_instruction: AtomicU8,
}

impl VM {
    pub fn init() -> VM {
        let mut vm = VM {
            classes: Mutex::new(vec![]),
            bootstrap_cl_class_list: Default::default(),
            object_arena: Default::default(),
            string_pool: Default::default(),

            object_class: ClassRef::new(null()),
            classloader: ClassRef::new(null()),
            string_class: ClassRef::new(null()),
            stat_file: Mutex::new(File::options().write(true).create(true).truncate(true)
                .open("stat.txt")
                .unwrap()),
            last_instruction: AtomicU8::new(0)
        };

        vm.load_bootstrap_classes();

        vm.object_class = ClassRef::new(&*vm.classes.lock().unwrap()[0]);
        vm.classloader = ClassRef::new(&*vm.classes.lock().unwrap()[1]);
        vm.string_class = ClassRef::new(&*vm.classes.lock().unwrap()[2]);

        vm
    }

    pub fn stop(&self) {
        println!("\n\n\nVM stats: ");
        println!("Loaded {} classes", self.bootstrap_cl_class_list.lock().unwrap().len());
        println!("Object arena allocated {} bytes of object",
                 self.object_arena.last_index.load(Ordering::Relaxed));
        println!("String pool has {} block(s) of data and has {} strings interned",
                 self.string_pool.buffers.read().unwrap().len(),
                 self.string_pool.interned_string.read().unwrap().len());
    }
}