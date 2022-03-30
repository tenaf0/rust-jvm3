use std::pin::Pin;
use std::ptr::null;
use once_cell::sync::OnceCell;
use smallvec::SmallVec;
use crate::class_parser::constants::CPTag::Methodref;
use crate::vm::class::class::{Class, ClassRepr};
use crate::vm::object::ObjectHeader;
use crate::vm::thread::thread::{MethodRef, VMThread};
use crate::vm::vm::VM;

mod class_parser;
mod vm;

static VM_HANDLER: OnceCell<VM> = OnceCell::new();

fn native_func(args: SmallVec<[u64; 32]>, exception: &mut Option<String>) -> Option<u64> {
    println!("This will run!");
    // *exception = Some("But will also throw an exception".to_string());
    return Some(5);
}

fn main() {
    let vm = VM_HANDLER.get_or_init(VM::init);

    let mut thread = VMThread::new();
    let handle = std::thread::spawn(move || {
        let c = &vm.classes.lock().unwrap()[0];
        println!("{:?}", c);

        thread.start(MethodRef::JVMMethod(c, 0));
    });

    /*let mut thread2 = VMThread::new();
    let handle2 = std::thread::spawn(move || {
        let c = &vm.classes.lock().unwrap()[0];
        println!("{:?}", c);

        thread2.start(MethodRef::NativeMethod(native_func));
    });*/

    handle.join();
    // handle2.join();

    /*let class = Class { header: ObjectHeader {
        class: unsafe { null() } },
        data: ClassRepr { name: "java/lang/Object".to_string() }
    };*/
}