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

    if t1.len() * t2.len() < 100000 {
        // Then compare the result with the standard dtw
        let distance2 = dtw.calculate(Box::new(t1.clone()), Box::new(t2.clone()));

        if distance2.0 > 0.0 {
            // Save the difference
            let diff = (100 * ((distance1.0 - distance2.0).abs() as i64)) / distance2.0 as i64;

            // Sum it to the global sum
            SUM_DIFF.fetch_add(diff, Ordering::SeqCst);

            let generated = COMPARED_DIFF.fetch_add(1, Ordering::SeqCst);

            if generated % 100 == 99 {
                let executed = SUM_DIFF.load(Ordering::SeqCst);
                log::debug!("Error rate {:.2}", executed as f32 / generated as f32,)
            }
        }
    }

    //
    // If state checking feature, then check for wasmtime instantiation of the binaries
});
