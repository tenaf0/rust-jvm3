use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::time::Duration;
use once_cell::sync::OnceCell;
use crate::main_loader::start_main_class;

use crate::vm::class::class::{Class, ClassRef, ClassRepr};
use crate::vm::class::field::FieldType;
use crate::vm::class::method::Method;
use crate::vm::class_loader::resolve::initialize_class;
use crate::vm::instructions::Instruction;
use crate::vm::object::ObjectHeader;
use crate::vm::thread::thread::{ThreadStatus, VMThread};
use crate::vm::vm::{VM, VmArgs};

mod class_parser;
mod vm;
mod helper;
mod main_loader;

static VM_HANDLER: OnceCell<VM> = OnceCell::new();

fn main() {
    use std::time::Instant;

    let now = Instant::now();
    let vm = VM_HANDLER.get_or_init(VM::init);

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

    let handle = std::thread::spawn(move || {
        start_main_class();
    });

    handle.join();
    vm.last_instruction.store(Instruction::impdep1 as u8, Ordering::Release);
    match stat_thread_handle {
        None => {},
        Some(handle) => { handle.join(); }
    }

    vm.stop();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}