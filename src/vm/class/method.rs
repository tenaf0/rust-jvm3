use std::fmt::{Debug, Formatter};
use smallvec::SmallVec;
use crate::class_parser::constants::AccessFlagMethod;
use crate::ClassRef;
use crate::helper::has_flag;
use crate::vm::class::field::FieldType;

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDescriptor {
    pub parameters: Vec<FieldType>,
    pub ret: FieldType
}

#[derive(Debug)]
pub struct Method {
    pub flag: u16,
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub repr: MethodRepr
}

impl Method {
    pub fn is_static(&self) -> bool {
        has_flag(self.flag, AccessFlagMethod::ACC_STATIC)
    }

    pub fn is_instance_init(&self, defining_class: ClassRef) -> bool {
        !defining_class.is_interface() && self.name == "<init>" && self.descriptor.ret == FieldType::V
    }
}

#[derive(Debug)]
pub enum MethodRepr {
    Jvm(JvmMethod),
    Native(NativeMethod)
}

#[derive(Debug)]
pub struct JvmMethod {
    pub code: Option<Code>,
}

pub const MAX_NO_OF_ARGS: usize = 64;

type NativeFnPtr = fn(SmallVec<[u64; MAX_NO_OF_ARGS]>, exception: &mut Option<String>) -> Option<u64>;

pub struct NativeMethod {
    pub fn_ptr: NativeFnPtr
}

impl Debug for NativeMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fn_ptr as usize)
    }
}

#[derive(Debug)]
pub struct Code {
    pub max_stack: usize,
    pub max_locals: usize,
    pub code: Vec<u8>
    // TODO: exception table, attributes
}