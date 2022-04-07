use std::mem::size_of;
use std::ptr::{null, null_mut};
use crate::Class;
use crate::vm::class::class::ClassRef;
use crate::vm::pool::string::StringObject;

/// Points to a variable length struct having the following data layout:
/// struct Object {
///     header: ObjectHeader,
///     data: [u64, FIELD_NO]
/// }
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectPtr {
    pub ptr: *mut u64
}

unsafe impl Sync for ObjectPtr {}
unsafe impl Send for ObjectPtr {}

impl ObjectPtr {
    pub fn null() -> ObjectPtr {
        ObjectPtr {
            ptr: null_mut()
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

        let mut ptr: *mut ObjectHeader = self.ptr.cast();
        ptr = unsafe { ptr.offset(1) };
        let mut ptr: *mut u64 = ptr.cast();

        // assert!(field_no >= 1 && field_no <= class.instance_field_count); TODO

        unsafe {
            ptr.offset(field_no as isize).read()
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(align(64))]
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