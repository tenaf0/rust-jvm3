use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use smallvec::SmallVec;
use crate::vm::class::constant_pool::CPEntry;
use crate::vm::class::field::Field;
use crate::vm::class::method::Method;
use crate::vm::object::ObjectHeader;

/// Runtime representation of a class in the method area, which is simultaneously has a correct
/// Object layout, so that Java objects can reference it (in Object::getClass for example)
#[derive(Debug)]
#[repr(C)]
pub struct Class {
    pub header: ObjectHeader,
    pub data: ClassRepr,
}

unsafe impl Sync for Class {}
unsafe impl Send for Class {}

#[derive(Debug)]
pub struct ClassRepr {
    pub name: String,
    // TODO: class_loader: ObjectRef,
    pub superclass: ClassRef,
    pub interfaces: SmallVec<[ClassRef; 32]>,
    pub constant_pool: Vec<CPEntry>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    // TODO: attributes
    pub static_fields: SmallVec<[u64; 32]>
}

/// Concrete type used as "pointer" to a Class instance
#[derive(Copy, Clone, Debug)]
pub struct ClassRef(pub *const Class);

unsafe impl Sync for ClassRef {}
unsafe impl Send for ClassRef {}