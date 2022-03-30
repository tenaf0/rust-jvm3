use crate::Class;

/// Points to a variable length struct having the following data layout:
/// struct Object {
///     header: ObjectHeader,
///     data: [u64, FIELD_NO]
/// }
#[derive(Copy, Clone, Debug)]
pub struct ObjectPtr {
    ptr: *mut u64
}

#[derive(Copy, Clone, Debug)]
#[repr(align(64))]
pub struct ObjectHeader {
    pub class: *const Class
}