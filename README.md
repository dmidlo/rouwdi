# rouwdi

rouwdi is defined as a complete Rust build chain packaged as one WebAssembly
assembly: `rouwdi.wasm`.

This repository is the new root product workspace. The reference Python repo
under `ref-repo/` is evidence only; the product boundary here is the WASM
assembly plus thin hosts that provide substrate.

Current implementation status: the root engine parses and validates
`rouwdi.toml`, snapshots source through a virtual storage interface, resolves a
Cargo workspace model without invoking host Cargo, handles virtual workspaces,
path/git/registry dependency source planning, contract-selected feature
resolution, frozen lockfile enforcement, vendored registry/git source
materialization into rouwdi-managed source cache, build scripts, proc-macro
targets, compile-time sandbox planning, internal compile-time WASM execution
for precompiled build-script and proc-macro modules, build graph planning,
manifest-relative Rust source paths, upstream `rustc_lexer` preflight proof
records, computed embedded target spec/ABI pack identities, per-target
interface/runtime proof records, and proof bundle verification. The native
runner is a thin Wasmtime/WASI substrate runner around `dist/rouwdi.wasm`; it
does not provide Cargo, rustc, a linker, target policy, validation, or proof
logic.

The repository also pins upstream compiler source custody with git submodules:

- `third_party/rust` at `800892799d7666fe1dc17abd862100a6cf273718`
- `third_party/rust/src/tools/cargo` at `a343accce8526b128adc517d33348573d22920a3`
- `third_party/rust/src/llvm-project` at `eaab4d9841b9a8a12783d927b2df2291c1c79269`

The build still refuses to report success until Rust compiler semantics,
codegen, linker, std/target/linker packs, build-script compilation into
compile-time WASM, and proc-macro crate compilation into compile-time WASM are
actually embedded inside `rouwdi.wasm`.

The refusal is intentional. rouwdi must not shell out to host `cargo`, `rustc`,
`lld`, or native build-script/proc-macro execution and call that complete.

## Commands

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p rouwdi-wasm --target wasm32-wasip1 --release
```

The WASI build command produces the assembly from `crates/rouwdi-wasm`. A
release packaging step may copy it to `dist/rouwdi.wasm`.
