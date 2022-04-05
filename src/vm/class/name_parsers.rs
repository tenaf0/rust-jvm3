use crate::vm::class::constant_pool::SymbolicReference::MethodReference;
use crate::vm::class::field::FieldType;
use crate::vm::class::method::MethodDescriptor;

impl MethodDescriptor {
    fn parse_field_type(str: &str, is_parameter: bool) -> (Option<FieldType>, &str) {
        match &str[0..1] {
            "B" => (Some(FieldType::B), &str[1..]),
            "C" => (Some(FieldType::C), &str[1..]),
            "D" => (Some(FieldType::D), &str[1..]),
            "F" => (Some(FieldType::F), &str[1..]),
            "I" => (Some(FieldType::I), &str[1..]),
            "J" => (Some(FieldType::J), &str[1..]),
            "L" => {
                let mut valid = true;
                let mut end = 0;
                for (i, c) in str[1..].chars().enumerate() {
                    if c == ')' {
                        valid = false;
                    } else if c == ';' {
                        end = i+1;
                        break;
                    }
                }

                if valid {
                    (Some(FieldType::L(str[1..end].to_string())),
                     &str[end+1..])
                } else {
                    (None, &str)
                }
            },
            "S" => (Some(FieldType::S), &str[1..]),
            "Z" => (Some(FieldType::Z), &str[1..]),
            "[" => {
                match Self::parse_field_type(&str[1..], true) {
                    (Some(component), rest) => (Some(FieldType::A(Box::new(component)))
                                                , rest),
                    _ => (None, &str)
                }
            },
            "V" =>  if is_parameter {
                        (None, &str)
                    } else {
                        (Some(FieldType::V), &str[1..])
                    }
            _ => (None, &str)
        }
    }

    pub fn parse(mut str: &str) -> Option<Self> {
        if &str[0..1] != "(" {
            return None;
        }

        str = &str[1..];

        let mut is_parsing_params = true;
        let mut parameters = vec![];
        while is_parsing_params {
            match Self::parse_field_type(str, true) {
                (Some(arg), rest) => {
                    parameters.push(arg);
                    str = rest;
                }
                _ => { is_parsing_params = false; }
            }
        }

        if &str[0..1] != ")" {
            return None;
        }

        str = &str[1..];

        let res = Self::parse_field_type(str, false);

        match res {
            (Some(ret), rest) if rest.is_empty() => Some(MethodDescriptor {
                parameters,
                ret,
            }),
            _ => None
        }
    }
}

impl FieldType {
    pub fn parse(str: &str) -> Option<Self> {
        MethodDescriptor::parse_field_type(str, true).0
    }
}

mod tests {
    use crate::vm::class::field::FieldType::*;
    use crate::vm::class::method::MethodDescriptor;

    #[test]
    fn parse_method_descriptor() {
        assert_eq!(MethodDescriptor::parse("()V"), Some(MethodDescriptor { parameters: vec![],
            ret: V }));

        assert_eq!(MethodDescriptor::parse("()[Ljava/lang/String;"), Some(MethodDescriptor { parameters: vec![],
            ret: A(Box::from(L(String::from("java/lang/String")))) }));

        assert_eq!(MethodDescriptor::parse("(IV)I"), None);
        assert_eq!(MethodDescriptor::parse("(I)I "), None);
        assert_eq!(MethodDescriptor::parse("(IJ[[Ljava/lang/String;)I"),
                   Some(MethodDescriptor {
                       parameters: vec![I, J, A(Box::new(A(Box::new(L(
                           String::from("java/lang/String"))))))],
                       ret: I
                   }));
    }
}
