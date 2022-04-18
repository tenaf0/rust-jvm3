use smallvec::smallvec;
use crate::{Class, ClassRef, FieldType, initialize_class, ThreadStatus, VM_HANDLER, VmArgs, VMThread};
use crate::vm::object::ObjectPtr;

pub fn start_main_class() {
    let vm = VM_HANDLER.get().unwrap();

    let arg = vm.args.read().unwrap();

    let main_class_name = vm.string_pool.intern_string(arg.main_class.as_str());

    let class_loader = vm.classloader;
    let mut loader_thread = VMThread::new();
    loader_thread.start((class_loader, 0), smallvec![0, main_class_name.to_val()]);
    match loader_thread.status {
        ThreadStatus::FINISHED(Some(res)) => {
            let main_class = ClassRef::new(res as *const Class);

            init_main_class(main_class);

            let main_method = main_class.data.methods.iter().enumerate().find(|(_i, m)| {
                m.name == "main" && m.is_static() && m.descriptor.ret == FieldType::V
                    && m.descriptor.parameters == vec![
                    FieldType::A(Box::new(FieldType::L("java/lang/String".to_string())))
                ]
            });

            let main_method = main_method.unwrap_or_else(|| panic!("No main method found"));
            let java_args: Vec<ObjectPtr> = arg.java_args.iter()
                .map(|s| vm.string_pool.add_string(s))
                .collect();

            let class = vm.load_class("[java/lang/String").unwrap();
            let array = vm.object_arena.new_array(class, java_args.len());
            for (i, ptr) in java_args.iter().enumerate() {
                array.store_to_array(i, ptr.to_val());
            }

            let mut main_thread = VMThread::new();
            main_thread.start((main_class, main_method.0), smallvec![array.to_val()]);
            match main_thread.status {
                ThreadStatus::FAILED(err) => println!("{}", err),
                _ => {}
            }
        }
        ThreadStatus::FAILED(err) => panic!("Could not load main class: {}", err),
        _ => panic!("Can't happen")
    }
}

fn init_main_class(class: ClassRef) {
    match initialize_class(class) {
        Ok(_) => {}
        Err(exc) => {
            panic!("Exception occurred while initializing main class: {}", exc)
        }
    }
}