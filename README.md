# RuSTRAC


Reimplementation of STRAC in Rust.

## Goals:

- To be faster than STRAC.
- CLI fully compilable to Wasm. Then we could use the argo Wasm integration to escalate pairwise comparison.
- File mapped memory to compare. Therefore, larger files.
