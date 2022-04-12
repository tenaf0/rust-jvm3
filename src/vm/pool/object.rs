use std::alloc;
use std::alloc::Layout;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use crate::{ClassRef, ObjectHeader};
use crate::vm::object::ObjectPtr;

#[derive(Debug)]
pub struct ObjectArena {
    last_index: AtomicUsize,
    arena: *mut AtomicU64,
    cap: usize
}

unsafe impl Sync for ObjectArena {}
unsafe impl Send for ObjectArena {}

const HEADER_SIZE: usize = std::mem::size_of::<ObjectHeader>();
const HEADER_ALIGN: usize = std::mem::align_of::<ObjectHeader>();

impl ObjectArena {
    fn calc_align(size: usize) -> usize {
        let size = HEADER_SIZE + size * std::mem::size_of::<AtomicU64>();
        let i = size / HEADER_ALIGN;

        let total_size = i * HEADER_ALIGN;

        if size > total_size {
            (total_size + HEADER_ALIGN) / std::mem::size_of::<AtomicU64>()
        } else {
            total_size / std::mem::size_of::<AtomicU64>()
        }
    }

    pub fn new() -> Self {
        let layout = Layout::array::<AtomicU64>(10*1024*1024).unwrap();
        let ptr = unsafe { alloc::alloc(layout) } as *mut AtomicU64;

        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        ObjectArena {
            last_index: AtomicUsize::new(0),
            arena: ptr,
            cap: Default::default()
        }
    }

    pub fn new_object(&self, class: ClassRef) -> ObjectPtr {
        self.allocate_object(class, class.data.instance_field_count)
    }

    pub fn new_array(&self, class: ClassRef, length: usize) -> ObjectPtr {
        let obj = self.allocate_object(class, length + 1);
        obj.put_field(0, length as u64);
        obj
    }

    fn allocate_object(&self, class: ClassRef, size: usize) -> ObjectPtr {
        let size = Self::calc_align(size);
        let offset = self.last_index.fetch_add(size, Ordering::AcqRel);

        println!("Allocating object of size {}", size);

        let ptr = unsafe { self.arena.offset(offset as isize ) };
        let mut header: *mut ObjectHeader = ptr.cast();
        unsafe { *header = ObjectHeader::new(class.ptr()); }
        header = unsafe { header.offset(1)};

        let field_ptr: *mut AtomicU64 = header.cast();
        for i in 0..size {
            unsafe { *field_ptr.offset(i as isize) = AtomicU64::new(0); }
        }

        ObjectPtr { ptr }
    }
}

impl Default for ObjectArena {
    fn default() -> Self {
        Self::new()
    }
}