use std::env;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::time::Duration;
use once_cell::sync::OnceCell;
use smallvec::{smallvec};

use crate::vm::class::class::{Class, ClassRef, ClassRepr};
use crate::vm::class::method::Method;
use crate::vm::class_loader::resolve::initialize_class;
use crate::vm::instructions::Instruction;
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

    if vm::thread::thread::ENABLE_STATS {
        // Start statistic thread
        std::thread::spawn(|| {
            use num_enum::FromPrimitive;

            let vm = VM_HANDLER.get().unwrap();
            let mut file = vm.stat_file.lock().unwrap();

            loop {
                let instruction = vm.last_instruction.load(Ordering::Acquire);
                let instruction = Instruction::from_primitive(instruction);
                let mut buf = Vec::with_capacity(16);
                write!(&mut buf, "{:?}\n", instruction);
                file.write(&buf);

                std::thread::sleep(Duration::from_millis(7));
            }
        });
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