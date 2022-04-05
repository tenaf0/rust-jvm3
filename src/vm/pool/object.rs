use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ObjectArena {
    pool: Mutex<Vec<u64>>,
    last_index: AtomicUsize
}

impl Default for ObjectArena {
    fn default() -> Self {
        ObjectArena {
            pool: Default::default(),
            last_index: AtomicUsize::new(0)
        }
    }
}