use std::ops::BitAnd;

pub fn has_flag<U, T: Into<U>>(value: U, flag: T) -> bool
    where U: BitAnd<Output = U> + PartialEq + Copy {
    let flag = flag.into();

    value & flag == flag
}

pub fn utof(u: u32) -> f32 {
    let ptr: *const u32 = &u;
    let ptr: *const f32 = ptr.cast();
    unsafe { *ptr }
}

pub fn ftou(f: f32) -> u32 {
    let ptr: *const f32 = &f;
    let ptr: *const u32 = ptr.cast();
    unsafe { *ptr }
}

pub fn utof2(u: u64) -> f64 {
    let ptr: *const u64 = &u;
    let ptr: *const f64 = ptr.cast();
    unsafe { *ptr }
}

pub fn ftou2(f: f64) -> u64 {
    let ptr: *const f64 = &f;
    let ptr: *const u64 = ptr.cast();
    unsafe { *ptr }
}

#[cfg(test)]
mod test {
    use crate::helper::{ftou2, utof2};

    #[test]
    pub fn float_converter() {
        let a: f64 = -3.141592654;
        let b = ftou2(a);
        let a2 = utof2(b);

        assert_eq!(a, a2);
    }
}