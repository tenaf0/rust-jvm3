use std::fmt::{Display, Formatter};
use std::io::{Cursor, Read};
use std::string::FromUtf8Error;
use crate::class_parser::types::{AttributeInfo, FieldInfo, MethodInfo, ParsedClass, U1, U2, U4};
use crate::class_parser::be_reader::BEReader;
use crate::class_parser::constants::{CPInfo, CPTag, tag_to_2U2_constructor, tag_to_2U4_constructor, tag_to_U2_constructor, tag_to_U4_constructor};

pub mod types;
pub mod constants;
mod be_reader;

#[derive(Debug)]
pub enum ParseErrorType {
    ParseErrorType
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorType,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

pub trait ToParseError {} // Required because generic would conflict with blanket impl of From

impl ToParseError for std::io::Error {}
impl ToParseError for FromUtf8Error {}

impl<T: Display + ToParseError> From<T> for ParseError {
    fn from(e: T) -> Self {
        ParseError {
            kind: ParseErrorType::ParseErrorType,
            message: e.to_string()
        }
    }
}

fn parse_cp_info(reader: &mut impl Read, constant_pool: &mut Vec<CPInfo>) -> Result<(), ParseError> {
    use CPTag::*;

    let tag = U1::read(reader)?;
    let tag = CPTag::try_from(tag)
        .map_err(|e| ParseError { kind: ParseErrorType::ParseErrorType, message: e.to_string() })?;

    match tag {
        Utf8 => {
            let length = U2::read(reader)? as usize;
            let mut buf = vec![0; length];
            reader.read_exact(&mut buf)?;
            constant_pool.push(CPInfo::Utf8(std::string::String::from_utf8(buf)?));
        }
        Class | String | MethodType | Module | Package => {
            let u2 = U2::read(reader)?;
            constant_pool.push(tag_to_U2_constructor(tag)(u2));
        }
        Integer | Float => {
            let u4 = U4::read(reader)?;
            constant_pool.push(tag_to_U4_constructor(tag)(u4));
        }
        Fieldref | Methodref | InterfaceMethodref | NameAndType | Dynamic | InvokeDynamic => {
            let u2_1 = U2::read(reader)?;
            let u2_2 = U2::read(reader)?;
            constant_pool.push(tag_to_2U2_constructor(tag)(u2_1, u2_2));
        }
        Long | Double => {
            let u4_1 = U4::read(reader)?;
            let u4_2 = U4::read(reader)?;
            constant_pool.push(tag_to_2U4_constructor(tag)(u4_1, u4_2));
            constant_pool.push(CPInfo::Hole);
        }
        MethodHandle => {
            let u1 = U1::read(reader)?;
            let u2 = U2::read(reader)?;
            constant_pool.push(CPInfo::MethodHandle(u1, u2));
        }
    }


    Ok(())
}

fn parse_field_info(reader: &mut impl Read) -> Result<FieldInfo, ParseError> {
    let access_flags = U2::read(reader)?;
    let name_index = U2::read(reader)?;
    let descriptor_index = U2::read(reader)?;
    let attributes_count = U2::read(reader)?;

    let mut attributes = Vec::with_capacity(attributes_count as usize);
    for _ in 0..attributes_count {
        attributes.push(parse_attributes_info(reader)?);
    }

    Ok(FieldInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes_count,
        attributes,
    })
}

fn parse_method_info(reader: &mut impl Read) -> Result<MethodInfo, ParseError> {
    let access_flags = U2::read(reader)?;
    let name_index = U2::read(reader)?;
    let descriptor_index = U2::read(reader)?;
    let attributes_count = U2::read(reader)?;

    let mut attributes = Vec::with_capacity(attributes_count as usize);
    for _ in 0..attributes_count {
        attributes.push(parse_attributes_info(reader)?);
    }

    Ok(MethodInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes_count,
        attributes,
    })
}

fn parse_attributes_info(reader: &mut impl Read) -> Result<AttributeInfo, ParseError> {
    let attribute_name_index = U2::read(reader)?;
    let attribute_length = U4::read(reader)?;

    let mut vec = vec![0; attribute_length as usize];
    reader.read_exact(&mut vec).unwrap();

    Ok(AttributeInfo {
        attribute_name_index,
        attribute_length,
        info: vec,
    })
}

// the method with the same name is unstable for cursor
trait ReadStatus {
    fn check_if_empty(&mut self) -> bool;
}
impl ReadStatus for Cursor<&[u8]> {
    fn check_if_empty(&mut self) -> bool {
        let mut buf = Vec::new();
        if let Ok(size) = self.read_to_end(&mut buf) {
            size == 0
        } else {
            false
        }
    }
}

pub fn parse_class(buf: &[u8]) -> Result<ParsedClass, ParseError> {
    let mut cursor = Cursor::new(buf);

    let magic = U4::read(&mut cursor)?;
    if magic != 0xCAFEBABE {
        return Err(ParseError { kind: ParseErrorType::ParseErrorType, message: "Not a class file"
            .to_string() });
    }

    let minor_version = U2::read(&mut cursor)?;
    let major_version = U2::read(&mut cursor)?;

    let constant_pool_count = U2::read(&mut cursor)? - 1;
    let mut constant_pool = Vec::with_capacity(constant_pool_count as usize);
    while constant_pool.len() < constant_pool_count as usize {
        parse_cp_info(&mut cursor, &mut constant_pool)?;
    }

    let access_flags = U2::read(&mut cursor)?;
    let this_class = U2::read(&mut cursor)?;
    let super_class = U2::read(&mut cursor)?;

    let interfaces_count = U2::read(&mut cursor)?;
    let mut interfaces = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        interfaces.push(U2::read(&mut cursor)?);
    }

    let fields_count = U2::read(&mut cursor)?;
    let mut fields = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        fields.push(parse_field_info(&mut cursor)?);
    }

    let methods_count = U2::read(&mut cursor)?;
    let mut methods = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        methods.push(parse_method_info(&mut cursor)?);
    }

    let attributes_count = U2::read(&mut cursor)?;
    let mut attributes = Vec::with_capacity(attributes_count as usize);
    for _ in 0..attributes_count {
        attributes.push(parse_attributes_info(&mut cursor)?);
    }



    if cursor.check_if_empty() {
        let parsed_class = ParsedClass {
            minor_version,
            major_version,
            constant_pool_count,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            methods_count,
            methods,
            attributes_count,
            attributes
        };

        // println!("{:#?}", parsed_class);
        Ok(parsed_class)
    } else {
        Err(ParseError {
            kind: ParseErrorType::ParseErrorType,
            message: "Class file is longer than expected".to_string()
        })
    }
}