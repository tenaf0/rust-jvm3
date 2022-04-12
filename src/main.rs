use std::env;
use once_cell::sync::OnceCell;
use smallvec::{smallvec};

use crate::vm::class::class::{Class, ClassRef, ClassRepr};
use crate::vm::class::method::Method;
use crate::vm::class_loader::resolve::initialize_class;
use crate::vm::object::ObjectHeader;
use crate::vm::thread::thread::{ThreadStatus, VMThread};
use crate::vm::vm::VM;

mod class_parser;
mod vm;
mod helper;

static VM_HANDLER: OnceCell<VM> = OnceCell::new();

fn main() {
    let vm = VM_HANDLER.get_or_init(VM::init);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please specify a class file to load");
        return;
    }

    let mut thread = VMThread::new();
    let handle = std::thread::spawn(move || {
        let c = vm.classloader;
        let ptr = vm.string_pool.intern_string(args[1].clone().as_str());

        thread.start((c, 0), smallvec![0, ptr.ptr as u64]);
        if let ThreadStatus::FINISHED(Some(class)) = thread.status {
            let ptr: *const Class = (class as *const u64).cast();
            let class = unsafe { ptr.read() };
            println!("Loaded: {}", class.data.name);

            if let Err(e) = initialize_class(ClassRef::new(ptr)) {
                eprintln!("Exception occured: {}", e);
                return;
            }

            let main_method = class.data.methods.iter().enumerate().find(|(_i, m)| {
                m.name == "main"
                // TODO: args
            });

            if let Some((i, _method)) = main_method {
                thread.start((ClassRef::new(ptr), i), smallvec![0]);
                println!("{:?}", thread.status);
            } else {
                println!("Class {} doesn't have a main method", class.data.name);
            }
        } else {
            println!("{:?}", thread.status);
        }
    });

    handle.join();

    vm.stop();
}