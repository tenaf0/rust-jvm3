use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;
use smallvec::{SmallVec};
use crate::ClassRef;
use crate::vm::class::method::MAX_NO_OF_ARGS;
use crate::vm::thread::thread::MethodRef;

const MAX_FRAME_SIZE: usize = 125;

pub struct Frame {
    pub methodref: MethodRef,
    pub pc: usize,
    local_array_size: usize,
    stack_size: usize,
    stack_top: usize,

    data: [MaybeUninit<u64>; MAX_FRAME_SIZE]
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[(")?;
        for i in 0..self.local_array_size {
            write!(f, "{}, ", unsafe { self.data[i].assume_init() })?;
        }
        write!(f, ") op: (")?;
        for i in 0..(self.stack_top - self.local_array_size) {
            write!(f, "{}, ", unsafe { self.data[self.local_array_size + i].assume_init() })?;
        }
        write!(f, ")]")
    }
}

impl Frame {
    pub fn new(methodref: MethodRef, local_array_size: usize, stack_size: usize) -> Self {
        assert!(local_array_size + stack_size <= MAX_FRAME_SIZE);

        Frame {
            methodref,
            pc: 0,
            local_array_size,
            stack_size,
            stack_top: local_array_size,
            // data: unsafe { MaybeUninit::uninit().assume_init() }
            data: [MaybeUninit::new(0); MAX_FRAME_SIZE] // TODO: For miri
        }
    }

    // Local array
    pub fn get_s(&self, index: usize) -> u32 {
        debug_assert!(index < self.local_array_size);

        (unsafe { self.data[index].assume_init() }) as u32
    }

    pub fn get_d(&self, index: usize) -> u64 {
        debug_assert!(index < self.local_array_size);

        unsafe { self.data[index].assume_init() }
    }

    pub fn set_s(&mut self, index: usize, val: u32) {
        debug_assert!(index < self.local_array_size);

        self.data[index] = MaybeUninit::new(val as u64);
    }

    pub fn set_d(&mut self, index: usize, val: u64) {
        debug_assert!(index < self.local_array_size);

        self.data[index] = MaybeUninit::new(val);
    }

    // Operand stack
    pub fn push(&mut self, val: u64) {
        debug_assert!(self.stack_top - self.local_array_size < self.stack_size);

        self.data[self.stack_top] = MaybeUninit::new(val);
        self.stack_top += 1;
    }

    pub fn pop(&mut self) -> u64 {
        debug_assert!(self.stack_top > self.local_array_size);

        self.stack_top -= 1;
        unsafe { self.data[self.stack_top].assume_init() }
    }

    pub fn clear_stack(&mut self)  {
        self.stack_top = self.local_array_size;
    }

    pub fn safe_peek(&self) -> Option<u64> {
        if self.stack_top > self.local_array_size {
            Some(unsafe { self.data[self.stack_top-1].assume_init() })
        } else {
            None
        }
    }

    pub fn pop_args(&mut self, no_of_args: usize) -> SmallVec<[u64; MAX_NO_OF_ARGS]> {
        let slice = unsafe { std::mem::transmute::<&[MaybeUninit<u64>], &[u64]>(
            &self.data[self.stack_top-no_of_args..self.stack_top]) };
        let args = SmallVec::from_slice(slice);

        self.stack_top -= no_of_args;

        args
    }

    pub fn peek_nth(&self, index: usize) -> u64 {
        unsafe { self.data[self.stack_top-index-1].assume_init() }
    }
}

mod test {
    use std::ptr::null;
    use crate::ClassRef;
    use crate::vm::thread::frame::Frame;

    #[test]
    fn test_peek() {
        let mut frame = Frame::new((ClassRef::new(null()), 0),2, 3);

        frame.push(1);
        frame.push(2);
        frame.push(3);

        assert_eq!(frame.peek_nth(0), 3);
        assert_eq!(frame.peek_nth(1), 2);

        frame.pop();

        assert_eq!(frame.peek_nth(1), 1);
    }
}

