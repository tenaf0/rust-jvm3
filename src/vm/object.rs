
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::Class;
use crate::vm::class::class::ClassRef;


/// Points to a variable length struct having the following data layout:
/// struct Object {
///     header: ObjectHeader,
///     data: [u64, FIELD_NO]
/// }
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectPtr {
    pub ptr: *const AtomicU64
}

unsafe impl Sync for ObjectPtr {}
unsafe impl Send for ObjectPtr {}

impl ObjectPtr {
    pub fn null() -> ObjectPtr {
        ObjectPtr {
            ptr: null_mut()
        }
    }

    pub fn from_val(val: u64) -> Option<ObjectPtr> {
        if val == 0 {
            None
        } else {
            Some(ObjectPtr {
                ptr: val as *const AtomicU64
            })
        }
    }

    pub fn get_class(&self) -> ClassRef {
        unsafe {
            let header: &ObjectHeader = &*self.ptr.cast();

            ClassRef::new(header.class)
        }
    }

    pub fn get_field(&self, field_no: usize) -> u64 {
        let class = self.get_class();
        assert!(field_no < class.data.instance_field_count);

        let mut ptr: *const ObjectHeader = self.ptr.cast();
        ptr = unsafe { ptr.offset(1) };
        let ptr: *const AtomicU64 = ptr.cast();

        unsafe {
            ptr.offset(field_no as isize).read().load(Ordering::Relaxed)
        }
    }

    pub fn put_field(&self, field_no: usize, val: u64) {
        let class = self.get_class();
        assert!(field_no < class.data.instance_field_count);

        let mut ptr: *const ObjectHeader = self.ptr.cast();
        ptr = unsafe { ptr.offset(1) };
        let ptr: *const AtomicU64 = ptr.cast();

        unsafe {
            (*ptr.offset(field_no as isize)).store(val, Ordering::Relaxed);
        }
    }
}

#[derive(Copy, Clone, Debug)]
// #[repr(align(64))]
pub struct ObjectHeader {
    pub class: *const Class,
    dummy_data: u32
}

impl ObjectHeader {
    pub fn new(ptr: *const Class) -> ObjectHeader {
        ObjectHeader {
            class: ptr,
            dummy_data: 0
        }
    }
}

impl Default for ObjectHeader {
    fn default() -> Self {
        ObjectHeader {
            class: null(),
            dummy_data: 0
        }
    }
}

unsafe impl Sync for ObjectHeader {}
unsafe impl Send for ObjectHeader {}

mod tests {
    use crate::{VM, VM_HANDLER};

    #[test]
    fn object_field() {
        let vm = VM_HANDLER.get_or_init(VM::init);

        let obj = vm.object_arena.new(vm.string_class);
        assert_eq!(obj.get_field(0), 0);

        obj.put_field(0, 3);
        assert_eq!(obj.get_field(0), 3);
    }
}