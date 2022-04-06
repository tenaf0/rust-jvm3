use std::pin::Pin;
use std::ptr::null;
use once_cell::sync::OnceCell;
use smallvec::{SmallVec, smallvec};
use crate::class_parser::constants::CPTag::Methodref;
use crate::vm::class::class::{Class, ClassRef, ClassRepr};
use crate::vm::class::method::Method;
use crate::vm::class_loader::resolve::initialize_class;
use crate::vm::object::ObjectHeader;
use crate::vm::thread::thread::{MethodRef, ThreadStatus, VMThread};
use crate::vm::vm::VM;

mod class_parser;
mod vm;
mod helper;

static VM_HANDLER: OnceCell<VM> = OnceCell::new();

fn main() {
    let vm = VM_HANDLER.get_or_init(VM::init);

    let mut thread = VMThread::new();
    let handle = std::thread::spawn(move || {
        let c = vm.classloader;
        println!("{:?}", c);

        let ptr = vm.string_pool.add_string("hu.garaba.Main");

        thread.start((c, 0), smallvec![0, ptr.ptr as u64]);
        if let ThreadStatus::FINISHED(Some(class)) = thread.status {
            let ptr: *const Class = (class as *const u64).cast();
            let class = unsafe { ptr.read() };
            println!("Loaded: {:#?}", class);

            if let Err(e) = initialize_class(ClassRef::new(ptr)) {
                eprintln!("Exception occured: {}", e);
                return;
            }

            let main_method = class.data.methods.iter().enumerate().find(|&m| match m {
                (i, Method::Jvm(method)) => method.name == "main",
                _ => false
            });

            if let Some((i, method)) = main_method {
                thread.start((ClassRef::new(ptr), i), smallvec![]);
            }
        }
    });

    handle.join();
}