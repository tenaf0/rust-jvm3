#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Field {}