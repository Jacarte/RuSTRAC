use crate::dtw::{Accesor, TokenID};
use std::sync::Arc;
use std::sync::Mutex;

pub struct MMapWrapper {
    pub size: usize,
    pub ptr: Arc<Mutex<*mut u8>>,
}

// This only works for UNIX like systems
// It is faster than the naive vectorization of the tokens :)
impl Accesor for MMapWrapper {
    fn get(&self, idx: usize) -> TokenID {
        // get 8 bytes from the pointer
        let ptr = unsafe { self.ptr.lock().unwrap().add(12 + idx * 8) };
        return unsafe { *ptr } as TokenID;
    }

    fn size(&self) -> usize {
        self.size
    }
}
