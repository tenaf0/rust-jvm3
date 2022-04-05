use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::{Mutex, RwLock};
use std::sync::atomic::AtomicU64;
use smallvec::SmallVec;
use crate::vm::class::constant_pool::CPEntry;
use crate::vm::class::field::Field;
use crate::vm::class::method::Method;
use crate::vm::object::ObjectHeader;

#[derive(Debug)]
pub enum ClassState {
    Loaded,
    Verified,
    Initializing,
    Ready
}

/// Runtime representation of a class in the method area, which is simultaneously has a correct
/// Object layout, so that Java objects can reference it (in Object::getClass for example)
#[derive(Debug)]
#[repr(C)]
pub struct Class {
    pub header: ObjectHeader,
    pub state: ClassState,
    pub data: ClassRepr,
}

unsafe impl Sync for Class {}
unsafe impl Send for Class {}

impl Class {
    pub fn get_cp_entry(&self, index: usize) -> &CPEntry {
        unsafe { &*self.data.constant_pool[index - 1].get() }
    }

    pub fn set_cp_entry(&self, index: usize, value: CPEntry) {
        // TODO: Create VM-wide lock for this
        unsafe { *self.data.constant_pool[index - 1].get() = value; }
    }
}

#[derive(Debug)]
pub struct ClassRepr {
    pub name: String,
    // TODO: class_loader: ObjectRef,
    pub superclass: ClassRef,
    pub interfaces: SmallVec<[ClassRef; 32]>,
    pub constant_pool: Vec<UnsafeCell<CPEntry>>, // TODO: make private
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    // TODO: attributes
    pub static_fields: SmallVec<[AtomicU64; 32]>
}

/// Concrete type used as "pointer" to a Class instance
#[derive(Copy, Clone, Debug)]
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