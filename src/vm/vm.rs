use std::pin::Pin;
use std::ptr::null;
use std::sync::Mutex;
use crate::{Class, ClassRepr, ObjectHeader};
use crate::vm::class::class::ClassRef;
use crate::vm::class::field::FieldType;
use crate::vm::class::method::{Code, Method, MethodDescriptor};

pub struct VM {
    pub classes: Mutex<Vec<Pin<Box<Class>>>> // Indirection is required due to pinning -- since
                                             // objects will point to the class directly, we
                                             // can't move them around
                                             // TODO: Allocate in special class area.
}

impl VM {
    pub fn init() -> VM {
        let zero_ptr = unsafe { null() };

        let mut vec = vec![];
        let ptr = Class {
            header: ObjectHeader { class: zero_ptr },
            data: ClassRepr {
                name: "java/lang/Object".to_string(),
                superclass: ClassRef(zero_ptr),
                interfaces: Default::default(),
                constant_pool: vec![],
                field_info: vec![],
                method_info: vec![
                    Method {
                        name: "main".to_string(),
                        descriptor: MethodDescriptor { parameters: vec![], ret: FieldType::V },
                        code: Some(Code {
                            max_stack: 2,
                            max_locals: 6,
                            code: vec![16, 14, 60, 17, 1, 158, 61, 16, 124, 62, 27, 28, 96, 54, 4, 27, 27, 96, 28, 96, 29, 96, 54, 5, 177]
                        })
                    }
                ],
                static_fields: Default::default()
            }
        };
        let pin1 = Box::pin(ptr);
        vec.push(pin1);

        VM {
            classes: Mutex::new(vec)
        }
    }
}