use std::alloc;
use std::alloc::Layout;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

use std::sync::RwLock;
use crate::{ObjectHeader, VM_HANDLER};
use crate::vm::object::ObjectPtr;

const STRING_FIELD_COUNT: usize = 1;
const STRING_POOL_BUFFER_SIZE: usize = 10;

#[derive(Debug)]
pub struct StringPool {
    pub buffers: RwLock<Vec<StrArena>>,
    pub interned_string: RwLock<HashMap<String, ObjectPtr>>
}

impl StringPool {
    pub fn add_string(&self, value: &str) -> ObjectPtr {
        let mut buffers = self.buffers.write().unwrap();
        let buffer = buffers.last_mut().unwrap();
        let option = buffer.add_string(value);

        match option {
            None => {
                buffers.push(StrArena::new());
                drop(buffers);

                println!("Creating new StrArena");
                self.add_string(value)
            }
            Some(res) => {
                res
            }
        }
    }

    pub fn intern_string(&self, value: &str) -> ObjectPtr {
        {
            let interned_map = self.interned_string.read().unwrap();
            if let Some(index) = interned_map.get(value) {
                return *index;
            }
        }

        let obj = self.add_string(value);

        eprintln!("Interned string {}", value);

        let mut interned_map = self.interned_string.write().unwrap();
        interned_map.insert(value.to_string(), obj);

        obj
    }
}

impl Default for StringPool {
    fn default() -> Self {
        StringPool {
            buffers: RwLock::new(vec![StrArena::new()]),
            interned_string: Default::default()
        }
    }
}

#[derive(Debug)]
pub struct StrArena {
    last_index: AtomicUsize,
    arena: *mut u8,
    cap: usize,
}

unsafe impl Send for StrArena {}
unsafe impl Sync for StrArena {}

impl StrArena {
    pub fn new() -> StrArena {
        const SIZE: usize = 1024;

        let layout = Layout::array::<u8>(SIZE).unwrap();
        let ptr = unsafe { alloc::alloc(layout) };

        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        StrArena {
            last_index: AtomicUsize::new(0),
            arena: ptr,
            cap: SIZE
        }
    }

    pub fn add_string(&self, str: &str) -> Option<ObjectPtr> {
        let str = str.as_bytes();
        let len = str.len();
        assert!(len < self.cap);

        let offset = self.last_index.fetch_add(len, Ordering::AcqRel);

        if offset + len >= self.cap {
            self.last_index.fetch_sub(len, Ordering::AcqRel);
            return None;
        }

        let ptr = unsafe { self.arena.offset(offset as isize) };

        unsafe { std::ptr::copy_nonoverlapping(str.as_ptr(), ptr, len) };

        let vm = VM_HANDLER.get().unwrap();
        let string = vm.object_arena.new_object(vm.string_class);
        string.put_field(0, len as u64);
        string.put_field(1, ptr as u64);

        Some(string)
    }

    pub fn get_string(obj: ObjectPtr) -> String {
        let vm = VM_HANDLER.get().unwrap();
        assert_eq!(obj.get_class(), vm.string_class);

        let length = obj.get_field(0) as usize;
        let ptr = obj.get_field(1) as *mut u8;

        if length == 0 {
            return String::new();
        }

        let str = unsafe { std::slice::from_raw_parts(ptr, length) };
        let str = Vec::from(str);
        String::from_utf8(str).unwrap()
    }
}

mod tests {
    use std::sync::atomic::Ordering;
    use crate::{VM, VM_HANDLER};
    use crate::vm::pool::string::{STRING_POOL_BUFFER_SIZE, StringPool};

    /*#[test]
    fn add_string() {
        let _vm = VM_HANDLER.get_or_init(VM::init);

        let pool = StringPool { buffers: Default::default(), interned_string: Default::default() };

        let ptr1 = pool.intern_string("string1");
        assert_eq!(pool.pool.read().unwrap().len(), 1);

        let ptr2 = pool.add_string("string2");
        let index = ptr2.get_field(0);
        assert_eq!(pool.pool.read().unwrap().len(), 2);
        assert_eq!(&*pool.get(index as usize), "string2");

        let ptr3 = pool.intern_string("string1");
        assert_eq!(pool.pool.read().unwrap().len(), 2);
        assert_eq!(pool.interned_string.read().unwrap().len(), 1);
        assert_eq!(ptr1.ptr, ptr3.ptr);

        let index = ptr3.get_field(0);
        assert_eq!(&*pool.get(index as usize), "string1");
    }*/
}