use std::ops::BitAnd;

pub fn has_flag<U, T: Into<U>>(value: U, flag: T) -> bool
    where U: BitAnd<Output = U> + PartialEq + Copy {
    let flag = flag.into();

    value & flag == flag
}

#[inline(always)]
pub fn utof(u: u64) -> f64 {
    let ptr: *const u64 = &u;
    let ptr: *const f64 = ptr.cast();
    unsafe { *ptr }
}

#[inline(always)]
pub fn ftou(f: f64) -> u64 {
    let ptr: *const f64 = &f;
    let ptr: *const u64 = ptr.cast();
    unsafe { *ptr }
}

mod test {
    use crate::helper::{ftou, utof};

    #[test]
    pub fn float_converter() {
        let a: f64 = -3.141592654;
        let b = ftou(a);
        let a2 = utof(b);

        assert_eq!(a, a2);
    }
}