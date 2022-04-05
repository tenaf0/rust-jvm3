use crate::class_parser::constants::{cp_info_to_tag, CPInfo, CPTag};

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: U2,
    pub name_index: U2,
    pub descriptor_index: U2,
    pub attributes_count: U2,
    pub attributes: Vec<AttributeInfo>, // of attributes_count length
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: U2,
    pub name_index: U2,
    pub descriptor_index: U2,
    pub attributes_count: U2,
    pub attributes: Vec<AttributeInfo>, // of attributes_count length
}

#[derive(Debug)]
pub struct AttributeInfo {
    pub attribute_name_index: U2,
    pub attribute_length: U4,
    pub info: Vec<u8>,
}

#[derive(Debug)]
pub struct ParsedClass {
    pub minor_version: U2,
    pub major_version: U2,
    pub constant_pool: Vec<CPInfo>, // of length constant_pool_count-1
    pub access_flags: U2,
    pub this_class: U2,
    pub super_class: U2,
    pub interfaces: Vec<U2>, // of length interfaces_count
    pub fields: Vec<FieldInfo>, // of length fields_count
    pub methods: Vec<MethodInfo>, // of length methods_count
    pub attributes: Vec<AttributeInfo>, // of length attributes_count
}

impl ParsedClass {
    pub fn get_cp_info_raw(&self, index: U2, cp_info_type: CPTag) -> Option<&CPInfo> {
        let elem = self.constant_pool.get(index as usize - 1)?;
        let tag = cp_info_to_tag(elem)?;
        if tag == cp_info_type {
            Some(elem)
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! get_cp_info {
    ($parsed_class: ident, $ind: expr, $tag: expr, $pat: pat, $param: expr) => {{
        let temp_cp_info = $parsed_class.get_cp_info_raw($ind, $tag)
                        .ok_or(format!("Format error at constant pool item {}",
                                                          $ind));
        match temp_cp_info {
            Ok(ok) => match ok {
                $pat => Ok($param),
                    _ => unreachable!()
            },
            Err(e) => Err(e)
        }
    }}
}