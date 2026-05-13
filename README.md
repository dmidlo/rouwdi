# rouwdi

rouwdi is defined as a complete Rust build chain packaged as one WebAssembly
assembly: `rouwdi.wasm`.

This repository is the new root product workspace. The reference Python repo
under `ref-repo/` is evidence only; the product boundary here is the WASM
assembly plus thin hosts that provide substrate.

Current implementation status: the root engine parses and validates
`rouwdi.toml`, snapshots source through a virtual storage interface, resolves a
Cargo workspace model without invoking host Cargo, writes first-class proof
bundle files, and refuses to report a successful build until Rust compiler,
codegen, linker, and target packs are actually embedded inside `rouwdi.wasm`.

The refusal is intentional. rouwdi must not shell out to host `cargo`, `rustc`,
`lld`, or native build-script/proc-macro execution and call that complete.

## Commands

```bash
cargo test --workspace
cargo build -p rouwdi-wasm --target wasm32-wasip1 --release
```

The WASI build command produces the assembly from `crates/rouwdi-wasm`. A
release packaging step may copy it to `dist/rouwdi.wasm`.

