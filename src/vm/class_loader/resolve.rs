

use std::thread;
use smallvec::smallvec;
use crate::{Class, ClassRef, VM_HANDLER, VMThread};
use crate::class_parser::constants::AccessFlagMethod;
use crate::helper::has_flag;
use crate::ThreadStatus::{FAILED, FINISHED};
use crate::vm::class::class::ClassState;
use crate::vm::class::constant_pool::{CPEntry, SymbolicReference, UnresolvedReference};

use crate::vm::class::constant_pool::UnresolvedReference::{ClassReference, FieldReference, MethodReference};
use crate::vm::class::field::{Field, FieldType};
use crate::vm::class::method::MethodRepr::Native;
use crate::vm::class::method::{NativeFnPtr, NativeMethod};
use crate::vm::class_loader::native::{init_native_store, NATIVE_FN_STORE, NativeMethodRef};

type Exception = String;

pub fn resolve(class: ClassRef, index: usize) -> Result<(), Exception> {
    use CPEntry::*;

    let entry = class.get_cp_entry(index);

    match entry {
        UnresolvedSymbolicReference(ClassReference(name)) => {
            let vm = VM_HANDLER.get().unwrap();

            let mut thread = VMThread::new();
            let classloader = vm.classloader;

            let ptr = vm.string_pool.intern_string(name);

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
        UnresolvedSymbolicReference(method @ MethodReference(class_index, _name, _descriptor)) => {
            resolve(class, *class_index as usize)?;

            // TODO: resolved class should *not* be an interface

            match class.get_cp_entry(*class_index as usize) {
                ResolvedSymbolicReference(SymbolicReference::ClassReference(other_class)) => {
                    let res = resolve_method(*other_class, method, false)?;

                    initialize_class(*other_class);

                    class.set_cp_entry(index, ResolvedSymbolicReference(res));

                    Ok(())
                }
                _ => panic!()
            }
        },
        UnresolvedSymbolicReference(FieldReference(class_index, name, descriptor)) => {
            resolve(class, *class_index as usize)?;

            match class.get_cp_entry(*class_index as usize) {
                ResolvedSymbolicReference(SymbolicReference::ClassReference(other_class)) => {
                    let mut instance_count = 0;
                    let mut static_count = 0;

                    let mut field= None;

                    for f in &other_class.data.fields {
                        if &f.name == name && &f.descriptor == descriptor {
                            field = Some(f);
                            break;
                        }

                        if f.is_static() {
                            static_count += 1;
                        } else {
                            instance_count += 1;
                        }
                    }

                    match field {
                        None => panic!("Could not find field"),
                        Some(field) => {
                            let field_index = if field.is_static() {
                                static_count
                            } else {
                                instance_count + other_class.data.superclass.data.instance_field_count
                            };
                            class.set_cp_entry(index, ResolvedSymbolicReference(
                                SymbolicReference::FieldReference(*other_class, !field.is_static(), field_index)));

                            initialize_class(*other_class);
                            // TODO: Recursive lookup
                        }
                    }
                }
                _ => panic!()
            }

            Ok(())
        },
        _ => Ok(())
    }
}

fn resolve_method(class: ClassRef, method: &UnresolvedReference, superclass: bool) ->
                                                                Result<SymbolicReference, Exception> {
    match method {
        MethodReference(_, name, descriptor) => {
            // signature-polymorph methods first
            if let Some((_, i)) = class.find_method(name, descriptor) {
                return Ok(SymbolicReference::MethodReference(class, i));
            }

            if !class.data.superclass.ptr().is_null() {
                if let Ok(res) = resolve_method(class.data.superclass, method, true) {
                    return Ok(res)
                }
            }

            if superclass {
                return Err(format!("No method found"));
            }

            // TODO: Implement interface lookup

            Err(format!("No method found: {:?}", method))
        }
        _ => panic!()
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


    eprintln!("Initializing {}", class.data.name);

    // TODO: Apply ConstantValue attribute
    let parent_list = vec![class.data.superclass];
    for _i in &class.data.interfaces {
        // TODO: Add to parent_list if declare at least one non-abstract, non-static method
    }

    for i in parent_list {
        initialize_class(i)?;
    }

    let clinit = class.data.methods.iter().enumerate().find(|(_i,m)| {
        m.name == "<clinit>"
            && m.descriptor.ret == FieldType::V
            && m.descriptor.parameters.len() == 0
            && m.is_static()
    });

    match clinit {
        Some((i, _)) => {
            let mut thread = VMThread::new();
            thread.start((class, i), smallvec![]);

            match thread.status {
                FINISHED(_) => *class.state.lock().unwrap() = ClassState::Ready,
                FAILED(e) => return Err(e),
                _ => panic!()
            }
        },
        None => {}
    }

    eprintln!("Initialized {}", class.data.name);
    Ok(())
}