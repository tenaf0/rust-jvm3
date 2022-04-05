use crate::class_parser::constants::AccessFlagField;
use crate::helper::has_flag;

#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    B,
    C,
    D,
    F,
    I,
    J,
    L(String),
    S,
    Z,
    A(Box<FieldType>), // [
    V
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub flag: u16,
    pub name: String,
    pub descriptor: FieldType
}

impl Field {
    pub fn is_static(&self) -> bool {
        has_flag(self.flag, AccessFlagField::ACC_STATIC)
    }
}