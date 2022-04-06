use std::fmt::format;
use std::thread;
use smallvec::smallvec;
use crate::{Class, ClassRef, Method, ThreadStatus, VM_HANDLER, VMThread};
use crate::ThreadStatus::{FAILED, FINISHED};
use crate::vm::class::class::ClassState;
use crate::vm::class::constant_pool::{CPEntry, SymbolicReference};
use crate::vm::class::constant_pool::UnresolvedReference::{ClassReference, FieldReference, MethodReference};
use crate::vm::class::field::{Field, FieldType};

type Exception = String;

pub fn resolve(class: ClassRef, index: usize) -> Result<(), Exception> {
    use CPEntry::*;

    let entry = class.get_cp_entry(index);

    match entry {
        UnresolvedSymbolicReference(ClassReference(name)) => {
            let vm = VM_HANDLER.get().unwrap();

            let mut thread = VMThread::new();
            let classloader = vm.classloader;

            let ptr = vm.string_pool.add_string(name);

            // Should be done by the defining loader of class, but currently we only support a
            // bootstrap class loader
            // TODO: Access control
            thread.start((classloader, 0), smallvec![0, ptr.ptr as u64]);
            match thread.status {
                FINISHED(Some(class_ptr)) => {
                    let class_ref = ClassRef::new(class_ptr as *const Class);
                    class.set_cp_entry(index, CPEntry::ResolvedSymbolicReference
                        (SymbolicReference::ClassReference(class_ref)));
                },
                FAILED(e) => return Err(e),
                _ => panic!("Can't happen")
            }

            Ok(())
        },
        UnresolvedSymbolicReference(MethodReference(class_index, name, descriptor)) => todo!(),
        UnresolvedSymbolicReference(FieldReference(class_index, name, descriptor)) => {
            resolve(class, *class_index as usize)?;

            match class.get_cp_entry(*class_index as usize) {
                ResolvedSymbolicReference(SymbolicReference::ClassReference(other_class)) => {
                    if let Some((field_index, _)) = other_class.data.fields.iter().enumerate()
                        .find(|(i, f)| &f.name == name && &f.descriptor == descriptor) {
                        class.set_cp_entry(index, ResolvedSymbolicReference(
                            SymbolicReference::FieldReference(class, false, field_index)));

                        initialize_class(*other_class);
                    }
                }
                _ => panic!()
            }

            Ok(())
        },
        _ => Ok(())
    }
}

pub fn initialize_class(class: ClassRef) -> Result<(), Exception> {
    {
        let class_state = &mut *class.state.lock().unwrap();
        if class_state == &ClassState::Ready ||
            class_state == &ClassState::Initializing(thread::current().id()) {
            return Ok(());
        } else if class_state != &ClassState::Verified {
            return Err(format!("Class '{}' should be in state Verified at initialization but it was \
        {:?}", class.data.name, class_state));
        }

        *class_state = ClassState::Initializing(thread::current().id());
    }


    println!("Initializing {}", class.data.name);

    let clinit = class.data.methods.iter().enumerate().find(|(i,m)| {
        match m {
            // TODO: Guard should have 'static' as well
            Method::Jvm(method) if method.name == "<clinit>" && method.descriptor.ret ==
                FieldType::V && method.descriptor.parameters.len() == 0 => true,
            _ => false
        }
    });

    match clinit {
        Some((i, _)) => {
            // TODO: Apply ConstantValue attribute
            let mut parent_list = vec![class.data.superclass];
            for i in &class.data.interfaces {
                // TODO: Add to parent_list if declare at least one non-abstract, non-static method
            }

            for i in parent_list {
                initialize_class(i)?;
            }

            let mut thread = VMThread::new();
            thread.start((class, i), smallvec![]);

            match thread.status {
                FINISHED(_) => *class.state.lock().unwrap() = ClassState::Ready,
                FAILED(e) => return Err(e),
                _ => panic!()
            }

            Ok(())
        },
        None => Err(format!("No <clinit> method was found in {}", class.data.name))
    }
}