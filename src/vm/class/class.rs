use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use num_enum::{FromPrimitive};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use smallvec::SmallVec;
use crate::class_parser::constants::AccessFlagClass;
use crate::helper::has_flag;
use crate::vm::class::constant_pool::CPEntry;
use crate::vm::class::field::Field;
use crate::vm::class::method::{Method, MethodDescriptor};
use crate::vm::object::{ObjectHeader, ObjectPtr};
use crate::vm::thread::thread::MethodRef;
use crate::VM_HANDLER;

#[derive(FromPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum ClassState {
    Loaded = 0,
    Verified = 1,
    Initializing = 2,
    #[default]
    Ready = 3,
}

pub struct AtomicClassState {
    pub state: AtomicU8
}

impl AtomicClassState {
    pub fn new(state: ClassState) -> AtomicClassState {
        AtomicClassState {
            state: AtomicU8::new(state as u8)
        }
    }

    pub fn get(&self) -> ClassState {
        let state = self.state.load(Ordering::Acquire);

        ClassState::from_primitive(state as u8)
    }

    pub fn set(&self, state: ClassState) {
        self.state.store(state as u8, Ordering::Release);
    }

    pub fn set_from(&self, from: ClassState, to: ClassState) -> Result<(), ()> {
        self.state.compare_exchange(from as u8, to as u8, Ordering::Release, Ordering::Relaxed)
            .map(|_| ()).map_err(|_| ())
    }
}

/// Runtime representation of a class in the method area, which is simultaneously has a correct
/// Object layout, so that Java objects can reference it (in Object::getClass for example)
#[repr(C)]
pub struct Class {
    pub header: ObjectHeader,
    pub state: AtomicClassState,
    pub data: ClassRepr,
}

impl Debug for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}]{:?}", self.state.get(), self.data)
    }
}

impl Class {
    pub fn is_interface(&self) -> bool {
        has_flag(self.data.flag, AccessFlagClass::ACC_INTERFACE)
    }

    pub fn is_subclass(&self, other: ClassRef) -> bool {
        let vm = VM_HANDLER.get().unwrap();
        if ClassRef::new(self) == other || other == vm.object_class {
            return true;
        }

        if !self.data.superclass.0.is_null() {
            return self.data.superclass.is_subclass(other);
        }

        false
    }

    pub fn find_method(&self, name: &str, descriptor: &MethodDescriptor) -> Option<MethodRef> {
        self.data.methods.iter().enumerate().find(|(_, m)| m.name == name
            && &m.descriptor == descriptor).map(|(i, _)| (ClassRef::new(self), i))
    }

    pub fn is_array(&self) -> bool {
        self.data.name.starts_with('[')
    }

    pub fn get_package(&self) -> (ObjectPtr, String) {
        let rightmost_slash = self.data.name.rfind('/');
        match rightmost_slash {
            None => (ObjectPtr::null(), "".to_string()),
            Some(i) => (ObjectPtr::null(), String::from(&self.data.name[0..i]))
        }
    }
}

unsafe impl Sync for Class {}
unsafe impl Send for Class {}

impl Class {
    pub fn get_cp_entry(&self, index: usize) -> &CPEntry {
        unsafe { &*self.data.constant_pool[index - 1].entry.get() }
    }

    pub fn set_cp_entry(&self, index: usize, value: CPEntry) {
        let _ = self.header.lock.lock().unwrap();

        unsafe { *self.data.constant_pool[index - 1].entry.get() = value; }
    }
}

#[derive(Debug)]
pub struct CPEntryWrapper {
    entry: UnsafeCell<CPEntry>
}

impl CPEntryWrapper {
    pub fn new(entry: &CPEntry) -> CPEntryWrapper {
        CPEntryWrapper {
            entry: UnsafeCell::new(entry.clone())
        }
    }
}

#[derive(Debug)]
pub struct ClassRepr {
    pub name: String,
    pub flag: u16,
    // TODO: class_loader: ObjectRef,
    pub superclass: ClassRef,
    pub interfaces: SmallVec<[ClassRef; 32]>,
    pub constant_pool: Vec<CPEntryWrapper>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    // TODO: attributes
    pub static_fields: SmallVec<[AtomicU64; 32]>,
    pub instance_field_count: usize // Cumulative size of all instance fields in the hierarchy
}

/// Concrete type used as "pointer" to a Class instance
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct ClassRef(*const Class);

impl ClassRef {
    pub fn new(ptr: *const Class) -> Self {
        ClassRef(ptr)
    }

    pub fn ptr(&self) -> *const Class {
        self.0
    }
}

impl Deref for ClassRef {
    type Target = Class;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

unsafe impl Sync for ClassRef {}
unsafe impl Send for ClassRef {}

mod tests {
    use std::sync::atomic::Ordering;
    use crate::{VM, VM_HANDLER};

    #[test]
    fn get_package_name() {
        let _vm = VM_HANDLER.get_or_init(| | VM::vm_init(false));

        assert_eq!("java/lang", _vm.object_class.get_package().1);
    }

    #[test]
    fn test_subclass_method() {
        let _vm = VM_HANDLER.get_or_init(| | VM::vm_init(false));

        let string = _vm.load_class("java/lang/String").unwrap();
        let object = _vm.load_class("java/lang/Object").unwrap();

        assert!(string.is_subclass(object));
    }
}