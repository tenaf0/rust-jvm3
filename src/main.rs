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
use crate::vm::vm::{VM};

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
    if cfg!(feature = "statistics") {
        // Start statistic thread
        stat_thread_handle = Some(std::thread::spawn(|| {
            use num_enum::FromPrimitive;

            let vm = VM_HANDLER.get().unwrap();

            let mut arr = [0; 256];

            loop {
                let instr = vm.last_instruction.load(Ordering::Acquire);
                let instruction = Instruction::from_primitive(instr);
                if instruction == Instruction::impdep1 {
                    break;
                }

                arr[instr as usize] += 1;

                std::thread::sleep(Duration::from_millis(1));
            }

            let mut file = File::options().write(true).create(true).truncate(true)
                .open("stat.txt")
                .unwrap();

            let mut buf = Vec::with_capacity(64);

            for i in 1..256 {
                if arr[i] == 0 {
                    continue;
                }

                let b = vm.instr_map[i].load(Ordering::Relaxed) as f64;
                let _ = write!(&mut buf, "{:?}, {}\n", Instruction::from_primitive(i as u8), (arr[i]
                    as f64) / b);
                let _ = file.write(&buf);
                buf.truncate(0);
            }
        }));
    } else {
        stat_thread_handle = None;
    }

    let handle = std::thread::spawn(move || {
        start_main_class();
    });

    let _ = handle.join();
    vm.last_instruction.store(Instruction::impdep1 as u8, Ordering::Release);
    match stat_thread_handle {
        None => {},
        Some(handle) => { let _ = handle.join(); }
    }

    vm.stop();
    let elapsed = now.elapsed();
    eprintln!("Elapsed: {:.2?}", elapsed);
}