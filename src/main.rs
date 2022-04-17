use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::time::Duration;
use once_cell::sync::OnceCell;
use smallvec::{smallvec};

use crate::vm::class::class::{Class, ClassRef, ClassRepr};
use crate::vm::class::field::FieldType;
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

    let stat_thread_handle;
    if vm::thread::thread::ENABLE_STATS {
        // Start statistic thread
        stat_thread_handle = Some(std::thread::spawn(|| {
            use num_enum::FromPrimitive;

            let vm = VM_HANDLER.get().unwrap();
            let mut file = vm.stat_file.lock().unwrap();
            if file.is_none() {
                *file = Some(File::options().write(true).create(true).truncate(true)
                    .open("stat.txt")
                    .unwrap());
            }

            let file = file.as_mut().unwrap();

            loop {
                let instruction = vm.last_instruction.load(Ordering::Acquire);
                let instruction = Instruction::from_primitive(instruction);
                if instruction == Instruction::impdep1 {
                    return;
                }

                let mut buf = Vec::with_capacity(16);
                write!(&mut buf, "{:?}\n", instruction);
                file.write(&buf);

                std::thread::sleep(Duration::from_millis(7));
            }
        }));
    } else {
        stat_thread_handle = None;
    }

    let mut thread = VMThread::new();
    let handle = std::thread::spawn(move || {
        let c = vm.classloader;
        let ptr = vm.string_pool.intern_string(args[1].clone().as_str());

        thread.start((c, 0), smallvec![0, ptr.to_val()]);
        if let ThreadStatus::FINISHED(Some(class)) = thread.status {
            let ptr: *const Class = (class as *const u64).cast();
            let class = unsafe { ptr.read() };
            eprintln!("Loaded: {}", class.data.name);

            if let Err(e) = initialize_class(ClassRef::new(ptr)) {
                eprintln!("Exception occured: {}", e);
                return;
            }

            let main_method = class.data.methods.iter().enumerate().find(|(_i, m)| {
                m.name == "main" && m.is_static() && m.descriptor.ret == FieldType::V
                    && m.descriptor.parameters == vec![FieldType::A(
                    Box::new(FieldType::L("java/lang/String".to_string())))]
            });

            if let Some((i, _method)) = main_method {
                let jvm_args = if args.len() > 2 {
                    let mut jvm_args = Vec::with_capacity(args.len() - 2);
                    for a in &args[2..] {
                        jvm_args.push(vm.string_pool.add_string(a));
                    }
                    jvm_args
                } else {
                    vec![]
                };

                let class = vm.load_class("[java/lang/String").unwrap();
                let array = vm.object_arena.new_array(class, jvm_args.len());
                for (i, ptr) in jvm_args.iter().enumerate() {
                    array.store_to_array(i, ptr.to_val());
                }

                let mut thread = VMThread::new();
                thread.start((ClassRef::new(ptr), i), smallvec![array.to_val()]);
                println!("{:?}", thread.status);
            } else {
                println!("Class {} doesn't have a main method", class.data.name);
            }
        } else {
            println!("{:?}", thread.status);
        }
    });

    handle.join();
    vm.last_instruction.store(Instruction::impdep1 as u8, Ordering::Release);
    match stat_thread_handle {
        None => {},
        Some(handle) => { handle.join(); }
    }

    vm.stop();
}