use std::collections::HashMap;

use std::sync::RwLock;
use crate::{ObjectHeader, VM_HANDLER};
use crate::vm::object::ObjectPtr;

const STRING_FIELD_COUNT: usize = 1;

#[derive(Debug)]
#[repr(C)]
pub struct StringObject {
    // This object can be referenced both as a Java object as well as a struct
    pub header: ObjectHeader,
    pub data: [u64; STRING_FIELD_COUNT],
    pub string: String
}

#[derive(Debug)]
pub struct StringPool {
    pool: RwLock<Vec<StringObject>>,
    interned_string: RwLock<HashMap<String, usize>>
}

impl StringPool {
    pub fn add_string(&self, value: &str) -> ObjectPtr {
        let index = self.add_string_to_pool(value);
        let mut pool = self.pool.write().unwrap();
        let ptr: *mut StringObject = pool.get_mut(index).unwrap();

        ObjectPtr { ptr: ptr.cast() }
    }

    fn add_string_to_pool(&self, value: &str) -> usize {
        let mut pool = self.pool.write().unwrap();
        let string_object = StringObject {
            header: ObjectHeader::new(VM_HANDLER.get().unwrap().string_class.ptr()),
            data: [pool.len() as u64],
            string: value.to_string()
        };

        pool.push(string_object);

        pool.len()-1
    }

    pub fn intern_string(&self, value: &str) -> ObjectPtr {
        {
            let interned_map = self.interned_string.read().unwrap();
            if let Some(index) = interned_map.get(value) {
                let ptr: *mut StringObject = self.pool.write().unwrap().get_mut(*index).unwrap();

                return ObjectPtr { ptr: ptr.cast() };
            }
        }


        let index = self.add_string_to_pool(value);
        let ptr: *mut StringObject = self.pool.write().unwrap().get_mut(index).unwrap();

        let mut interned_map = self.interned_string.write().unwrap();
        interned_map.insert(value.to_string(), index);

        ObjectPtr { ptr: ptr.cast() }
    }

    pub fn get(&self, index: usize) -> String {
        let pool = self.pool.read().unwrap();

        pool[index].string.clone()
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

mod tests {
    use std::sync::atomic::Ordering;
    use crate::{VM, VM_HANDLER};
    use crate::vm::pool::string::StringPool;

    #[test]
    fn add_string() {
        let _vm = VM_HANDLER.get_or_init(VM::init);

        let pool = StringPool { pool: Default::default(), interned_string: Default::default() };

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
    }
}