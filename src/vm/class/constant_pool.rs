use crate::class_parser::constants::CPInfo;
use crate::vm::class::class::ClassRef;
use crate::vm::class::field::{Field, FieldType};
use crate::vm::class::method::{Method, MethodDescriptor};
use crate::vm::object::ObjectPtr;

#[derive(Debug)]
pub enum CPEntry {
    UnresolvedSymbolicReference(UnresolvedReference),
    ResolvedSymbolicReference(SymbolicReference),
    ConstantString(ObjectPtr),
    ConstantValue(u64), // Integer, long, float, double value
    Hole
}

#[derive(Debug)]
pub enum UnresolvedReference {
    ClassReference(String),
    MethodReference(u16, String, MethodDescriptor), // class reference index, method name, method descriptor
    FieldReference(u16, String, FieldType)
}

#[derive(Debug)]
pub enum SymbolicReference {
    ClassReference(ClassRef),
    MethodReference(*const Method),
    FieldReference(*const Field)
}