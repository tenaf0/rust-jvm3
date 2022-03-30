use crate::vm::class::field::FieldType;

#[derive(Debug, PartialEq)]
pub struct MethodDescriptor {
    pub parameters: Vec<FieldType>,
    pub ret: FieldType
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub code: Option<Code>,
}

#[derive(Debug)]
pub struct Code {
    pub max_stack: usize,
    pub max_locals: usize,
    pub code: Vec<u8>
    // TODO: exception table, attributes
}