use std::fmt::format;
use std::ptr::null;
use std::sync::Mutex;
use crate::{Class, ClassRef, ClassRepr, ObjectHeader};
use crate::vm::class::class::ClassState;
use crate::vm::class::field::FieldType;

pub fn create_primitive_array_class(component: FieldType) -> Option<Class> {
    match component {
        FieldType::L(_) | FieldType::A(_) | FieldType::V => return None,
        _ => {}
    }

    Some(Class {
        header: ObjectHeader::default(),
        state: Mutex::new(ClassState::Ready),
        data: ClassRepr {
            name: format!("[{:?}", component),
            flag: 0,
            superclass: ClassRef::new(null()),
            interfaces: Default::default(),
            constant_pool: vec![],
            fields: vec![],
            methods: vec![],
            static_fields: Default::default(),
            instance_field_count: 0
        }
    })
}