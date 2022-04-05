use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;
use crate::{ObjectHeader, VM_HANDLER};
use crate::vm::object::ObjectPtr;

const STRING_FIELD_COUNT: usize = 1;

#[derive(Debug)]
#[repr(C)]
pub struct StringObject {
    // This object can be referenced both as a Java object as well as a struct
    pub(crate) header: ObjectHeader,
    pub data: [u64; STRING_FIELD_COUNT],
    pub string: String
}

#[derive(Debug)]
pub struct StringPool {
    pool: RwLock<Vec<StringObject>>,
    interned_string: RwLock<HashMap<String, ObjectPtr>>
}

impl StringPool {
    pub fn add_string(&self, value: &str) -> ObjectPtr {
        let mut pool = self.pool.write().unwrap();
        let mut string_object = StringObject {
            header: ObjectHeader { class: VM_HANDLER.get().unwrap().string_class.ptr() },
            data: [pool.len() as u64],
            string: value.to_string()
        };

        println!("{:?}", string_object);
        pool.push(string_object);
        let ptr: *mut StringObject = pool.last_mut().unwrap();

        ObjectPtr { ptr: unsafe { ptr.cast() } }
    }

    pub fn get(&self, index: usize) -> *const String {
        let mut pool = self.pool.read().unwrap();

        &pool[index].string
    }
}

impl Default for StringPool {
    fn default() -> Self {
        StringPool {
            pool: Default::default(),
            interned_string: Default::default()
        }
    }
}