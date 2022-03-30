use crate::Class;
use crate::vm::object::ObjectPtr;

pub mod class;
mod constant_pool;
pub mod field;
pub mod method;

/*#[derive(Debug)]
pub enum CPEntry {
    ClassInfo(ClassReference),
    FieldInfo(FieldReference),
    MethodInfo(MethodReference),
    StaticConst(StaticConst),
    Empty
}*/

#[derive(Debug)]
pub enum ClassReference {
    Unresolved(String), // TODO: Parse array syntax
    Resolved(*const Class), // TODO: Parse array syntax
}

#[derive(Debug)]
enum FieldReference {
    Unresolved(String, FieldDescriptor, usize),
}

/*#[derive(Debug)]
pub enum MethodReference {
    Unresolved(usize, String, MethodDescriptor),
}*/

#[derive(Debug)]
struct FieldDescriptor;

#[derive(Debug)]
pub enum StaticConst {
    String(ObjectPtr), // Reference to String instance
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64)
}