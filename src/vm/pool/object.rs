use std::alloc;
use std::alloc::Layout;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use crate::{ClassRef, ObjectHeader};
use crate::vm::object::ObjectPtr;

#[derive(Debug)]
pub struct ObjectArena {
    last_index: AtomicUsize,
    lock: Mutex<bool>,
    arena: *mut AtomicU64,
    cap: AtomicUsize
}

unsafe impl Sync for ObjectArena {}
unsafe impl Send for ObjectArena {}

impl ObjectArena {
    pub fn new(&self, class: ClassRef) -> ObjectPtr {
        self.lock.lock().unwrap();

        let mut val = self.last_index.load(Ordering::Relaxed) as isize;
        let ptr = unsafe { self.arena.offset(val ) };
        let mut header: *mut ObjectHeader = ptr.cast();
        unsafe { *header = ObjectHeader::new(class.ptr()); }
        header = unsafe { header.offset(1)};
        let field_ptr: *mut AtomicU64 = header.cast();

        for i in 0..class.data.instance_field_count {
            unsafe { *field_ptr.offset(i as isize) = AtomicU64::new(0);}
        }

        let i = (std::mem::size_of::<ObjectHeader>() / 8) as isize;
        val += class.data.instance_field_count as isize + i;
        self.last_index.store(val as usize, Ordering::Relaxed);

        ObjectPtr { ptr }
    }
}

impl Default for ObjectArena {
    fn default() -> Self {
        let layout = Layout::array::<AtomicU64>(1000*256).unwrap();
        let ptr = unsafe { alloc::alloc(layout) } as *mut AtomicU64;

        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        ObjectArena {
            last_index: AtomicUsize::new(0),
            lock: Mutex::new(false),
            arena: ptr,
            cap: Default::default()
        }
    }
}