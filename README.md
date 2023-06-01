# RuSTRAC


Reimplementation of STRAC in Rust.

## How to use it

- Download the binary that better suits you from the release page. Then run `dtw-tools --help`.
- Run `cargo run...`

## Compiling and testing

- Run `cargo test` for testing.
- Run `cargo bench` for benching the different implementations.
- Run `cargo build` for compiling.

## Roadmap

- [x] Traditional DTW
- [x] Processing of generic files using endline as the separator between trace tokens
- [x] Memoized DTW
- [x] CLI tool
- [x] Generic discrete cost function
- [ ] Wavefront implementation for SIMD
- [ ] Generic tokens separator
- [ ] Generic token filter
- [ ] FastDTW
- [ ] Export alignment
- [ ] Writing the trace in a custom bin file for faster reading.

## Goals:

- To be faster than STRAC.
- CLI fully compilable to Wasm. Then we could use the argo Wasm integration to escalate pairwise comparison.
- File mapped memory to compare. Therefore, larger files.
