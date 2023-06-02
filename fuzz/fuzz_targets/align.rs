#![no_main]
use arbitrary::{Arbitrary, Error, Unstructured};
use dtw::dtw::*;
use libfuzzer_sys::*;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::{fmt::Debug, fs};

pub static COMPARED_DIFF: AtomicI64 = AtomicI64::new(0);
pub static SUM_DIFF: AtomicI64 = AtomicI64::new(0);

fuzz_target!(|data: (Vec<usize>, Vec<usize>, usize, usize)| {
    let _ = env_logger::try_init();

    let t1 = data.0;
    let t2 = data.1;

    // swap if t2 > t1
    let (t1, t2) = if t1.len() > t2.len() {
        (t2, t1)
    } else {
        (t1, t2)
    };

    let rad = data.2.max(2);
    let th = data.3.max(10);

    log::debug!("t1 len {}", t1.len());
    log::debug!("t2 len {}", t2.len());
    log::debug!("rad {}", rad);
    log::debug!("th {}", th);

    let start = std::time::Instant::now();
    let distance = STRACDistance::default();
    let dtw = StandardDTW::new(&distance);

    let fastdtw = FastDTW::new(&distance, rad, th, &dtw);

    let distance1 = fastdtw.calculate(Box::new(t1.clone()), Box::new(t2.clone()));

    log::debug!("Result {}", distance1.0);

    //
    // If state checking feature, then check for wasmtime instantiation of the binaries
});
