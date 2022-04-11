use crate::class_parser::constants::AccessFlagField;
use crate::helper::has_flag;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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

impl FieldType {
    pub fn convert_newarray_type(index: u8) -> &'static str {
        match index {
            4 => "[Z",
            5 => "[C",
            6 => "[F",
            7 => "[D",
            8 => "[B",
            9 => "[S",
            10 => "[I",
            11 => "[J",
            _ => panic!()
        }
    }
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