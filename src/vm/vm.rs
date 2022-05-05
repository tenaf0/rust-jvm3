use std::collections::HashMap;
use std::pin::Pin;
use std::ptr::null;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Mutex, RwLock};

use clap::Parser;

use crate::{Class};
use crate::vm::class::class::ClassRef;
use crate::vm::pool::object::ObjectArena;
use crate::vm::pool::string::StringPool;

pub struct VM {
    pub args: RwLock<VmArgs>,

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

    pub last_instruction: AtomicU8,
    // #[cfg(feature = "statistics")]
    pub instr_map: [AtomicU64; 256]
}

#[derive(Parser, Debug)]
#[clap(about)]
pub struct VmArgs {
    #[clap(long = "cp")]
    pub classpath: Option<String>,
    pub main_class: String,
    pub java_args: Vec<String>,

    #[clap(long)]
    pub print_trace: bool
}

impl VM {
    pub fn init() -> VM {
        VM::vm_init(true)
    }

    pub fn vm_init(parse_args: bool) -> VM {
        const ZERO: AtomicU64 = AtomicU64::new(0);

        let mut vm = VM {
            args: RwLock::new(if parse_args { VmArgs::parse() } else { VmArgs {
                classpath: None,
                main_class: "".to_string(),
                java_args: vec![],
                print_trace: false
            } }),
            classes: Mutex::new(vec![]),
            bootstrap_cl_class_list: Default::default(),
            object_arena: Default::default(),
            string_pool: Default::default(),

            object_class: ClassRef::new(null()),
            classloader: ClassRef::new(null()),
            string_class: ClassRef::new(null()),
            last_instruction: AtomicU8::new(0),
            instr_map: [ZERO; 256]
        };

        vm.load_bootstrap_classes();

        vm.object_class = ClassRef::new(&*vm.classes.lock().unwrap()[0]);
        vm.classloader = ClassRef::new(&*vm.classes.lock().unwrap()[1]);
        vm.string_class = ClassRef::new(&*vm.classes.lock().unwrap()[2]);

        vm
    }

    pub fn stop(&self) {
        eprintln!("\n\n\nVM stats: ");
        eprintln!("Loaded {} classes", self.bootstrap_cl_class_list.lock().unwrap().len());
        eprintln!("Object arena allocated {} bytes of object",
                 self.object_arena.last_index.load(Ordering::Relaxed));
        eprintln!("String pool has {} block(s) of data and has {} strings interned",
                 self.string_pool.buffers.read().unwrap().len(),
                 self.string_pool.interned_string.read().unwrap().len());
    }
}