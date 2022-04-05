use std::fmt::{Debug, Formatter};
use smallvec::SmallVec;
use crate::vm::class::field::FieldType;

#[derive(Debug, PartialEq)]
pub struct MethodDescriptor {
    pub parameters: Vec<FieldType>,
    pub ret: FieldType
}

#[derive(Debug)]
pub enum Method {
    Jvm(JvmMethod),
    Native(NativeMethod)
}

#[derive(Debug)]
pub struct JvmMethod {
    pub name: String,
    pub descriptor: MethodDescriptor,
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