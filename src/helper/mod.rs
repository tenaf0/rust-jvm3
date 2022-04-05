use std::ops::BitAnd;

pub fn has_flag<U, T: Into<U>>(value: U, flag: T) -> bool
    where U: BitAnd<Output = U> + PartialEq + Copy {
    let flag = flag.into();

    value & flag == flag
}