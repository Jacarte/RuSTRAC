// use criterion::measurement::*;
use criterion::*;
use dtw::dtw::*;
use std::time::Duration;

fn bench_dtws(c: &mut Criterion) {
    let mut group = c.benchmark_group("dtws");

    // 0 is for dtw standard
    group.bench_function(BenchmarkId::new("dtw/1000", 0), |b| {
        b.iter_custom(|iters| {
            let mut cumulative_duration = Duration::new(0, 0);
            // Create two arrays full of random elements
            let x: Vec<usize> = (0..1000).map(|_| rand::random()).collect();
            let y: Vec<usize> = (0..1000).map(|_| rand::random()).collect();

            for _ in 0..iters {
                let start = std::time::Instant::now();
                // We include the instantiation of the DTWImpl in the benchmark
                let distancefunc = dtw::dtw::STRACDistance::default();
                let dtw = dtw::dtw::StandardDTW::new(&distancefunc);
                let _distance = dtw.calculate(Box::new(x.clone()), Box::new(y.clone()));

                cumulative_duration += start.elapsed();
            }

            cumulative_duration
        })
    });

    group.bench_function(BenchmarkId::new("unsafe dtw/1000", 0), |b| {
        b.iter_custom(|iters| {
            let mut cumulative_duration = Duration::new(0, 0);
            // Create two arrays full of random elements
            let x: Vec<usize> = (0..1000).map(|_| rand::random()).collect();
            let y: Vec<usize> = (0..1000).map(|_| rand::random()).collect();

            for _ in 0..iters {
                let start = std::time::Instant::now();
                let distancefunc = dtw::dtw::STRACDistance::default();

                // We include the instantiation of the DTWImpl in the benchmark
                let dtw = dtw::dtw::UnsafeDTW::new(&distancefunc);
                let _distance = dtw.calculate(Box::new(x.clone()), Box::new(y.clone()));

                cumulative_duration += start.elapsed();
            }

            cumulative_duration
        })
    });

    group.bench_function(BenchmarkId::new("fixed dtw/1000", 0), |b| {
        b.iter_custom(|iters| {
            let mut cumulative_duration = Duration::new(0, 0);
            // Create two arrays full of random elements
            let x: Vec<usize> = (0..1000).map(|_| rand::random()).collect();
            let y: Vec<usize> = (0..1000).map(|_| rand::random()).collect();

            for _ in 0..iters {
                let start = std::time::Instant::now();
                let distancefunc = dtw::dtw::STRACDistance::default();
                // We include the instantiation of the DTWImpl in the benchmark
                let dtw = dtw::dtw::FixedDTW::new(&distancefunc);
                let _distance = dtw.calculate(Box::new(x.clone()), Box::new(y.clone()));

                cumulative_duration += start.elapsed();
            }

            cumulative_duration
        })
    });

    group.bench_function(BenchmarkId::new("fastdtw/1000", 0), |b| {
        b.iter_custom(|iters| {
            let mut cumulative_duration = Duration::new(0, 0);
            // Create two arrays full of random elements
            let x: Vec<usize> = (0..1000).map(|_| rand::random()).collect();
            let y: Vec<usize> = (0..1000).map(|_| rand::random()).collect();

            for _ in 0..iters {
                let start = std::time::Instant::now();
                let distance = STRACDistance::default();
                let dtw = StandardDTW::new(&distance);

                let fastdtw = FastDTW::new(&distance, 2, 100, &dtw);

                let _distance = fastdtw.calculate(Box::new(x.clone()), Box::new(y.clone()));

                cumulative_duration += start.elapsed();
            }

            cumulative_duration
        })
    });
}

criterion_group!(benches, bench_dtws);
criterion_main!(benches);
