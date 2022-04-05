use smallvec::smallvec;
use crate::{Class, ClassRef, ThreadStatus, VM_HANDLER, VMThread};
use crate::ThreadStatus::{FAILED, FINISHED};
use crate::vm::class::constant_pool::{CPEntry, SymbolicReference};
use crate::vm::class::constant_pool::UnresolvedReference::{ClassReference, FieldReference, MethodReference};
use crate::vm::class::field::Field;

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
                    }
                }
                _ => panic!()
            }

            Ok(())
        },
        _ => Ok(())
    }
}