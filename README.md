# RuSTRAC


Reimplementation of STRAC in Rust.

## How to use it

- Download the binary that better suits you from the release page. Then run `dtw-tools --help`.
- Run `cargo run...`

## Compiling and testing

- Run `cargo test -p dtw` for testing.
- Run `cargo bench -p dtw` for benching the different implementations.
- Run `cargo build` for compiling.
- Run `cargo build --target=wasm32-wasi` to create a Wasm-WASI binary with the DTW implementations.

## Roadmap

- [x] Traditional DTW
- [x] Processing of generic files using endline as the separator between trace tokens
- [x] Memoized DTW
- [x] CLI tool
- [x] Generic discrete cost function
- [ ] Wavefront implementation for SIMD
- [ ] Generic tokens separator
- [ ] Generic token filter
- [x] FastDTW
- [x] Export alignment
- [x] Writing the trace in a custom bin file for faster reading.
- [x] Clippy and fmt in CI
- [ ] Automatic package deploy in cargo
- [ ] Doc generation
- [ ] Errorify ?

## Goals:

- To be faster than STRAC.
- CLI fully compilable to Wasm. Then we could use the argo Wasm integration to escalate pairwise comparison.
- File mapped memory to compare. Therefore, larger files.
