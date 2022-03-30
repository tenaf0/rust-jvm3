use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::class_parser::types::{U1, U2, U4};

#[derive(TryFromPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum CPTag {
    Utf8 = 1,
    Integer = 3,        // START loadable
    Float = 4,
    Long = 5,
    Double = 6,
    Class = 7,
    String = 8,         // END
    Fieldref = 9,
    Methodref = 10,
    InterfaceMethodref = 11,
    NameAndType = 12,
    MethodHandle = 15,  // START loadable
    MethodType = 16,
    Dynamic = 17,       // END
    InvokeDynamic = 18,
    Module = 19,
    Package = 20,
}

pub fn tag_to_U2_constructor(tag: CPTag) -> fn(U2) -> CPInfo {
    match tag {
        CPTag::Class => CPInfo::Class,
        CPTag::String => CPInfo::String,
        CPTag::MethodType => CPInfo::MethodType,
        CPTag::Module => CPInfo::Module,
        CPTag::Package => CPInfo::Package,
        _ => panic!("Tag should not be matched by this function!")
    }
}

pub fn tag_to_U4_constructor(tag: CPTag) -> fn(U4) -> CPInfo {
    match tag {
        CPTag::Integer => CPInfo::Integer,
        CPTag::Float => CPInfo::Float,
        _ => panic!("Tag should not be matched by this function!")
    }
}

pub fn tag_to_2U2_constructor(tag: CPTag) -> fn(U2, U2) -> CPInfo {
    match tag {
        CPTag::Fieldref => CPInfo::Fieldref,
        CPTag::Methodref => CPInfo::Methodref,
        CPTag::InterfaceMethodref => CPInfo::InterfaceMethodref,
        CPTag::NameAndType => CPInfo::NameAndType,
        CPTag::Dynamic => CPInfo::Dynamic,
        CPTag::InvokeDynamic => CPInfo::InvokeDynamic,
        _ => panic!("Tag should not be matched by this function!")
    }
}

pub fn tag_to_2U4_constructor(tag: CPTag) -> fn(U4, U4) -> CPInfo {
    match tag {
        CPTag::Long => CPInfo::Long,
        CPTag::Double => CPInfo::Double,
        _ => panic!("Tag should not be matched by this function!")
    }
}

pub const fn cp_info_to_tag(cp_info: &CPInfo) -> Option<CPTag> {
    match cp_info {
        CPInfo::Class(_) => Some(CPTag::Class),
        CPInfo::String(_) => Some(CPTag::String),
        CPInfo::MethodType(_) => Some(CPTag::MethodType),
        CPInfo::Module(_) => Some(CPTag::Module),
        CPInfo::Package(_) => Some(CPTag::Package),
        CPInfo::Integer(_) => Some(CPTag::Integer),
        CPInfo::Float(_) => Some(CPTag::Float),
        CPInfo::MethodHandle(_, _) => Some(CPTag::MethodHandle),
        CPInfo::Fieldref(_, _) => Some(CPTag::Fieldref),
        CPInfo::Methodref(_, _) => Some(CPTag::Methodref),
        CPInfo::InterfaceMethodref(_, _) => Some(CPTag::InterfaceMethodref),
        CPInfo::NameAndType(_, _) => Some(CPTag::NameAndType),
        CPInfo::Dynamic(_, _) => Some(CPTag::Dynamic),
        CPInfo::InvokeDynamic(_, _) => Some(CPTag::InvokeDynamic),
        CPInfo::Long(_, _) => Some(CPTag::Long),
        CPInfo::Double(_, _) => Some(CPTag::Double),
        CPInfo::Utf8(_) => Some(CPTag::Utf8),
        CPInfo::Hole => None
    }
}

#[derive(Debug)]
pub enum CPInfo {
    Class(U2),
    String(U2),
    MethodType(U2),
    Module(U2),
    Package(U2),
    Integer(U4),
    Float(U4),
    MethodHandle(U1, U2),
    Fieldref(U2, U2),
    Methodref(U2, U2),
    InterfaceMethodref(U2, U2),
    NameAndType(U2, U2),
    Dynamic(U2, U2),
    InvokeDynamic(U2, U2),
    Long(U4, U4),
    Double(U4, U4),
    Utf8(std::string::String),
    Hole, // Used for marking an empty slot in the constant pool (for long, double)
}

/*pub enum AccessFlagClass {
    ACC_PUBLIC = 0x0001,
    ACC_FINAL = 0x0010,
    ACC_SUPER = 0x0020,
    ACC_INTERFACE = 0x0200,
    ACC_ABSTRACT = 0x0400,
    ACC_SYNTHETIC = 0x1000,
    ACC_ANNOTATION = 0x2000,
    ACC_ENUM = 0x4000,
    ACC_MODULE = 0x8000,
}

#[derive(IntoPrimitive)]
#[repr(u16)]
pub enum AccessFlagField {
    ACC_PUBLIC = 0x0001,
    ACC_PRIVATE = 0x0002,
    ACC_PROTECTED = 0x0004,
    ACC_STATIC = 0x0008,
    ACC_FINAL = 0x0010,
    ACC_VOLATILE = 0x0040,
    ACC_TRANSIENT = 0x0080,
    ACC_SYNTHETIC = 0x1000,
    ACC_ENUM = 0x4000,
}

pub enum AccessFlagMethod {
    ACC_PUBLIC = 0x0001,
    ACC_PRIVATE = 0x0002,
    ACC_PROTECTED = 0x0004,
    ACC_STATIC = 0x0008,
    ACC_FINAL = 0x0010,
    ACC_SYNCHRONIZED = 0x0020,
    ACC_BRIDGE = 0x0040,
    ACC_VARARGS = 0x0080,
    ACC_NATIVE = 0x0100,
    ACC_ABSTRACT = 0x0400,
    ACC_STRICT = 0x0800,
    ACC_SYNTHETIC = 0x1000,
}*/