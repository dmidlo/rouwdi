# rouwdi — final room summation, with misconceptions removed

This is the cleaned, stricter, no-backoff version.

The project is **not**:

```text id="03fxtp"
a Python build service
a CLI around cargo
a wrapper around rustc
a Temporal workflow
a REST/MCP control plane
a Rust-like toy compiler
a Cranelift demo
a staged MVP compiler
a browser UI around native tools
a partial planner that shells out to the real build chain
```

The project is:

```text id="9mabxv"
a complete Rust build chain packaged into one WebAssembly assembly:

    rouwdi.wasm
```

The current prototype repo, `wasm-rust-builder`, has already proven important pieces of the idea: contract-driven native/WASI dual-target build, Rust stage1 custody, artifacts, logs, manifests, witness/proof surfaces, and a branch-owned compile lane. Its README says the repo proves a native stage1 Rust toolchain that can compile Rust crates to `native_host` and `wasm32-wasip1`, while also saying the Python service is the proved reference implementation rather than the final shipped object. 

That is the correct interpretation:

```text id="b3leyr"
wasm-rust-builder = proving ground / reference implementation / evidence
rouwdi.wasm        = final product
```

---

# 1. The final definition

**rouwdi is a full Rust build-chain assembly distributed as a single WebAssembly artifact.**

It contains everything required to build a real Rust application:

```text id="2kvxv7"
Cargo-compatible project resolution
Cargo.toml parsing
workspace resolution
feature resolution
lockfile handling
registry/git dependency fetching
crate cache
build graph planning
build.rs execution
proc macro execution
rustc frontend semantics
macro expansion
name resolution
type checking
borrow checking
MIR
monomorphization
codegen
object/archive emission
linking
WASI target support
native target support
std/core/alloc/proc_macro target packs
interface validation
runtime validation
hashing
proof bundle generation
```

The host does **not** provide Cargo.

The host does **not** provide rustc.

The host does **not** provide a linker.

The host does **not** decide targets.

The host does **not** validate artifacts.

The host does **not** write the proof bundle.

The host provides only substrate:

```text id="m9w2mg"
storage
network
clock
entropy
worker/thread scheduling
stdout/stderr/events
artifact export/import
browser/native runtime embedding
```

The build chain lives inside:

```text id="g2rjyl"
rouwdi.wasm
```

---

# 2. The shortest true sentence

> **rouwdi is the complete Rust build chain as one WebAssembly assembly.**

Slightly fuller:

> **rouwdi is a self-contained Rust build/proof engine packaged as `rouwdi.wasm`: it resolves Cargo projects, compiles Rust, links native and WASI artifacts, runs compile-time code in a sandbox, and emits proof bundles, while hosts provide only primitive runtime capabilities like storage and network.**

The product tagline:

> **Build Rust anywhere. Emit native + WASI. Prove the bytes.**

---

# 3. The corrected architecture

The old architecture was too weak:

```text id="e0tluo"
rouwdi.wasm:
  build planner
  proof engine

host:
  cargo
  rustc
  linker
  wasmtime
```

That is not enough.

That would make rouwdi portable only in appearance. The host would still be the real build chain.

The corrected architecture is:

```text id="50virl"
rouwdi.wasm:
  Cargo
  rustc
  compiler driver
  build graph
  build script sandbox
  proc macro sandbox
  codegen
  linker
  target packs
  proof engine

host:
  storage
  network
  runtime embedding
  UI
  artifact I/O
```

Diagram:

```text id="f6iztt"
┌───────────────────────────────────────────────────────────────────┐
│                           rouwdi.wasm                              │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Cargo-compatible build system                               │  │
│  │  - Cargo.toml parser                                        │  │
│  │  - workspace/package graph                                  │  │
│  │  - feature resolver                                         │  │
│  │  - lockfile model                                           │  │
│  │  - registry/git source resolver                             │  │
│  │  - crate cache                                               │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Compile-time execution layer                                │  │
│  │  - build.rs sandbox                                          │  │
│  │  - proc-macro sandbox                                        │  │
│  │  - OUT_DIR model                                             │  │
│  │  - Cargo env/config model                                    │  │
│  │  - compile-time stdout instruction parser                    │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Rust compiler layer                                          │  │
│  │  - parser                                                    │  │
│  │  - macro expansion                                           │  │
│  │  - name resolution                                           │  │
│  │  - type checking                                             │  │
│  │  - borrow checking                                           │  │
│  │  - MIR                                                       │  │
│  │  - metadata                                                  │  │
│  │  - monomorphization                                          │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Codegen and linker layer                                     │  │
│  │  - LLVM-grade release backend                                │  │
│  │  - object emission                                           │  │
│  │  - archive/staticlib creation                                │  │
│  │  - wasm linker                                               │  │
│  │  - native linker                                             │  │
│  │  - target ABI packs                                          │  │
│  │  - std/core/alloc/proc_macro packs                           │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Proof layer                                                  │  │
│  │  - source snapshot proof                                     │  │
│  │  - dependency graph proof                                    │  │
│  │  - build graph proof                                         │  │
│  │  - toolchain identity proof                                  │  │
│  │  - artifact interface proof                                  │  │
│  │  - runtime behavior proof                                    │  │
│  │  - hash report                                               │  │
│  │  - proof bundle writer                                       │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
              │                       │                    │
              ▼                       ▼                    ▼
        host storage             host network          host runtime
```

The host is boring.

The assembly is the build chain.

---

# 4. The non-negotiable boundary

Inside `rouwdi.wasm`:

```text id="ar93m8"
Cargo-compatible resolver
crate source fetch planner
registry/git source model
lockfile resolver
package graph
feature resolver
build graph scheduler
build.rs compiler/executor
proc macro compiler/executor
rustc frontend
rustc query model
MIR
codegen
linker
target packs
std/core/alloc/proc_macro packs
WASI support
native object/executable emission
proof bundle writer
```

Outside `rouwdi.wasm`:

```text id="jyaon4"
Wasmtime/Wasmer/browser runtime
filesystem/storage implementation
network implementation
clock/entropy
parallel worker substrate
terminal/browser/CI UI
artifact import/export
```

Absolutely not outside:

```text id="pjeuzl"
cargo
rustc
lld
wasm-ld
host-native proc macro dylibs
host-native build.rs execution
target decision logic
artifact validation
proof generation
```

That is the line.

---

# 5. The current prototype’s real contribution

`wasm-rust-builder` already has a lot of the **proof discipline** and **contract discipline**.

It currently has:

```text id="ba5x5o"
source-based Rust stage1 toolchain custody
job/request models
artifacts
logs
manifests
witness events
native_host target
wasm32-wasip1 target
branch-owned compile contract
runtime proof
interface proof
hash/proof surfaces
```

The existing `wrb-compile.toml` is basically the ancestor of `rouwdi.toml`. It declares a Rust package/bin/profile, a `wasm32-wasip1` target with required `_start`, a runtime expectation under Wasmtime, and a `native_host` target with executable validation and runtime proof. 

The existing `BranchWasmCompileService` already behaves like the reference version of rouwdi’s build/proof engine: it resolves the toolchain, resolves a committed source ref, materializes a detached worktree, loads the compile contract, builds each target, validates interfaces, executes runtime proofs, finalizes a compile manifest, and records per-target artifacts/proofs. 

The test suite already proves the important behavior: successful dual-target compile, native and WASI artifacts, interface proofs, execution proofs, source worktree cleanup, missing contract failure, missing ref failure, ambiguous primary target rejection, missing WASM output failure, missing native output failure, missing export failure, runtime timeout failure, exit-code mismatch failure, stdout mismatch failure, and dirty working tree isolation. 

That means the prototype is not disposable.

It is the specification seed.

But it is not the final architecture.

The current repo also contains too much control-plane weight: FastAPI, gRPC, MCP, Temporal, Textual, Uvicorn, and service dependencies show up directly in the package metadata. 

The Makefile also shows the sprawl: release gates, Temporal tests, republic proof, corpus proof, mailbox proof, bridge proof, proof export, adjacent audit, telemetry, freeze proof, panels, and more. 

That stuff is not rouwdi.

It may remain in the lab.

But the product is:

```text id="lcstcu"
single full build-chain assembly: rouwdi.wasm
```

---

# 6. What dies completely

These framings are dead:

## Dead framing 1

```text id="xx5sy0"
rouwdi is a small MVP compiler.
```

No.

That is too small and wrong.

## Dead framing 2

```text id="0o05ou"
rouwdi is a CLI that shells out to cargo/rustc.
```

No.

That leaves the build chain on the host.

## Dead framing 3

```text id="dgv6ic"
rouwdi is a planner/proof engine around external toolchains.
```

No.

That is a portability trap.

## Dead framing 4

```text id="z3bqor"
rouwdi is a Python service made smaller.
```

No.

The Python repo is a reference/prototype.

## Dead framing 5

```text id="mfw4ac"
Cranelift is the rouwdi backend story.
```

No.

Cranelift can be optional, experimental, or debug-only, but it is not the rouwdi identity. The rouwdi story is full-featured Rust output, not “fast-ish partial backend.” No lame backend center.

## Dead framing 6

```text id="ij1e0w"
Browser support comes later.
```

No.

Browser liftability is one of the reasons the whole thing must be one WASM build chain.

## Dead framing 7

```text id="l4u3qs"
The host wrapper can be smart.
```

No.

A smart wrapper recreates the old problem.

The host is substrate.

The assembly is the system.

---

# 7. What survives

These are the durable truths:

```text id="deeitx"
rouwdi is rouwdi.wasm.
```

```text id="e1ox31"
rouwdi.wasm contains the complete build chain.
```

```text id="69mge9"
The host does not provide Cargo, rustc, linker, or build policy.
```

```text id="uxkk7b"
The host provides storage/network/runtime primitives only.
```

```text id="rfa3l1"
The contract is checked in as rouwdi.toml.
```

```text id="zbnzo0"
A Rust source snapshot plus rouwdi.toml produces native and WASI artifacts.
```

```text id="8cjrsq"
Every emitted artifact gets interface, runtime, and hash proof.
```

```text id="wp6l6n"
Build scripts and proc macros run inside rouwdi’s sandbox, not as host-native code.
```

```text id="f0bs7o"
The browser path is real because the entire build chain is inside the WASM assembly.
```

```text id="b8qiiu"
The existing prototype is the behavioral oracle, not the final package.
```

---

# 8. The actual product stack

The full product stack is:

```text id="sj6e7b"
rouwdi.wasm
  the complete build-chain assembly

rouwdi
  tiny native runner that embeds or loads rouwdi.wasm

rouwdi-web
  browser host that runs rouwdi.wasm

rouwdi-ci
  CI host that runs rouwdi.wasm

rouwdi.toml
  checked-in build contract

.rouwdi/
  local state, cache, proof bundles, artifacts
```

Native:

```text id="3l56lg"
$ rouwdi build
```

really means:

```text id="0c6ip6"
native runner
  loads rouwdi.wasm
  grants storage/network/runtime primitives
  invokes rouwdi.wasm::build()
```

Browser:

```text id="8y8f3k"
web app
  loads rouwdi.wasm
  grants OPFS/storage/fetch/worker primitives
  invokes rouwdi.wasm::build()
```

CI:

```text id="mwiy5l"
CI action
  loads rouwdi.wasm
  grants workspace/cache/network/output primitives
  invokes rouwdi.wasm::build()
```

Same assembly.

Different host.

Same build semantics.

Same proof format.

---

# 9. The single-file implication

The statement “full-featured in a single `rouwdi.wasm`” has consequences.

It means `rouwdi.wasm` is not a tiny coordinator.

It is large.

It may include compressed internal packs.

It may include internal component sections.

It may contain embedded data tables for target specs, std artifacts, compiler metadata, linker pieces, and runtime proof schemas.

Externally, though, the user-facing artifact is one thing:

```text id="zsfbkt"
rouwdi.wasm
```

A strict packaging model:

```text id="kyg4h9"
dist/
  rouwdi.wasm
  rouwdi                 # optional tiny runner
  checksums.txt
```

Not:

```text id="5y18pa"
dist/
  rouwdi.wasm
  cargo
  rustc
  linker
  target-packs/
  random sidecars
```

The full-featured release can still have editions:

```text id="hx7fyo"
rouwdi.wasm              # standard full build chain
rouwdi-minimal.wasm      # maybe later, but not the main identity
rouwdi-nightly.wasm      # experimental
```

But the canonical claim remains:

```text id="g0slx1"
one rouwdi.wasm contains the build chain
```

---

# 10. What “full Rust application” means

A “full Rust application” does not mean only:

```text id="c7h8zl"
src/main.rs
Cargo.toml
cargo build
```

It means all the stuff real Rust projects use:

```text id="8glea1"
workspaces
multiple packages
features
target-specific dependencies
build dependencies
dev dependencies where relevant
build.rs
OUT_DIR
env-driven build config
proc-macro crates
macro expansion
generated code
cfg flags
target specs
std/core/alloc/proc_macro
linker args
native/static libraries where modeled
WASI libc/CRT where needed
archives
final executable/module/component emission
```

So rouwdi cannot dodge:

```text id="3ll0yz"
build.rs
proc macros
linking
std target support
dependency resolution
```

Those are the hard parts.

They are also what make the project real.

---

# 11. Build scripts inside rouwdi

Build scripts are not optional.

Many Rust crates use them.

Therefore:

```text id="4c4vh2"
build.rs must compile and execute inside rouwdi.wasm.
```

Not on the host.

Not through a native subprocess.

Not through host Cargo.

The rouwdi model:

```text id="vck8u7"
build.rs source
  ↓
rouwdi internal compile-time target
  ↓
compile-time WASM module
  ↓
rouwdi sandbox executes it
  ↓
stdout cargo directives parsed internally
  ↓
build graph updated internally
```

The sandbox must model:

```text id="bt7qc4"
OUT_DIR
CARGO_MANIFEST_DIR
TARGET
HOST
PROFILE
DEP_* variables
rerun-if-changed
rerun-if-env-changed
rustc-link-lib
rustc-link-search
rustc-cfg
rustc-env
warning/error lines
generated files
```

Build scripts do not get arbitrary host power.

They get a rouwdi-controlled sandbox.

This is necessary for browser liftability and reproducibility.

---

# 12. Procedural macros inside rouwdi

Proc macros are also not optional.

A huge amount of real Rust uses:

```text id="pb85az"
serde derive
tokio macros
clap derive
thiserror
async-trait
tracing attributes
diesel macros
custom derive crates
```

So:

```text id="gdv1nd"
proc-macro crates must compile and execute inside rouwdi.wasm.
```

Not as host-native dynamic libraries.

Not through host rustc.

The rouwdi model:

```text id="d6k929"
proc-macro crate source
  ↓
compile to rouwdi internal compile-time WASM ABI
  ↓
load into proc-macro sandbox
  ↓
pass token streams
  ↓
receive generated token streams
  ↓
continue compiler pipeline
```

This implies rouwdi needs an internal proc-macro ABI.

It does not need to expose this to users initially, but internally it must exist.

The proof bundle should record proc-macro identity:

```text id="tw59fu"
proc macro crate name
version/source hash
compiled macro module hash
macro expansion dependency edge
```

---

# 13. Cargo inside rouwdi

rouwdi must contain a Cargo-compatible build system.

Not a Cargo-inspired subset.

Cargo compatibility means:

```text id="ybo81o"
Cargo.toml parsing
workspace member discovery
package selection
target selection
features v2 behavior
dependency graph resolution
lockfile reading/writing
registry source model
git source model
path dependencies
build dependencies
proc macro dependencies
target-specific dependencies
profile handling
artifact naming
target directory layout or compatible logical equivalent
```

The internal command is not:

```text id="f9jvgc"
toolchain.run("cargo", ...)
```

The internal flow is:

```text id="tijwx5"
rouwdi.cargo.resolve()
rouwdi.cargo.plan()
rouwdi.compiler.compile_unit()
rouwdi.linker.link()
rouwdi.proof.emit()
```

The host may provide HTTP fetches and storage.

But the dependency resolver and build planner live inside the assembly.

---

# 14. rustc inside rouwdi

This is the central commitment.

rouwdi must contain Rust compiler semantics.

It should not invent a Rust-like compiler.

It should not implement a partial language.

It should not say “real Rust later.”

The target is:

```text id="vi7r1j"
real Rust source in
real Rust artifacts out
```

That means rouwdi needs the equivalent of:

```text id="g4jc26"
parse
expand
resolve
typeck
borrowck
lower to MIR
monomorphize
metadata
codegen
```

The sane conceptual route is not “write a new Rust compiler from scratch.”

The sane route is:

```text id="jc0gz4"
WASM-package the Rust compiler/build chain itself.
```

The final product identity is:

```text id="mi14g3"
Rust build chain, sealed into WebAssembly.
```

Not:

```text id="chmoti"
new almost-Rust compiler
```

---

# 15. Codegen: no lame center

Cranelift is not the identity.

The rouwdi codegen story is:

```text id="0e6b7s"
LLVM-grade release output
```

Cranelift may exist as:

```text id="kzpk8t"
optional debug backend
experimental backend
fast local analysis backend
maybe never surfaced publicly
```

But the core release story is not Cranelift.

The internal backend model should be pluggable, but the default must be production-grade:

```text id="s30zpk"
[backend]
default = "llvm-grade"

available:
  llvm-grade
  maybe-gcc
  maybe-cranelift-debug
```

The public README should not lead with Cranelift.

It should lead with:

```text id="h86k1p"
full Rust build chain in WASM
```

---

# 16. Linking inside rouwdi

A full Rust application does not stop at object files.

rouwdi needs to produce final artifacts.

For WASI:

```text id="uf28h7"
.wasm module
.wasm component where applicable
WASI imports
self-contained runtime pieces
_start or component exports
```

For native:

```text id="2j059t"
object files
archives
static libs
final executable where target pack supports it
```

The linker must be inside rouwdi.

Not host `ld`.

Not host `lld`.

The host may write bytes to disk.

But the linking decision and linking operation are rouwdi responsibilities.

Target packs carry the needed target-specific pieces:

```text id="t4pitk"
target spec
ABI model
crt objects
libstd/core/alloc/proc_macro artifacts
linker scripts/config
libc/WASI libc where applicable
archive format support
object format support
```

---

# 17. Target packs

Because the entire build chain is inside `rouwdi.wasm`, target packs cannot be “go install this host toolchain.”

They must be part of the assembly or internally loadable from rouwdi-managed storage.

For the strict single-file distribution:

```text id="ihpbpf"
target packs are embedded into rouwdi.wasm
```

At minimum:

```text id="06u0l0"
wasm32-wasip1
wasm32-wasip2
native_host family support
```

For a broader release:

```text id="s2rn9d"
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-pc-windows-msvc or gnu
wasm32-wasip1
wasm32-wasip2
wasm32-unknown-unknown
```

The proof bundle records:

```text id="nnkgl7"
target pack id
target pack hash
std pack hash
linker pack hash
compiler engine hash
```

This matters because if rouwdi is the build chain, the build chain itself must be part of the proof.

---

# 18. Browser liftability

The browser point is not cosmetic.

It is the forcing function.

Because `rouwdi.wasm` contains the full build chain, browser mode can be real:

```text id="qsnyk4"
browser loads rouwdi.wasm
browser gives storage
browser gives network fetch
browser gives worker parallelism
rouwdi.wasm resolves crates
rouwdi.wasm compiles Rust
rouwdi.wasm links artifacts
rouwdi.wasm writes proof bundle
```

Browser host substrate:

```text id="jxxzuo"
OPFS or equivalent origin storage
Fetch API wrapper
Web Workers
IndexedDB/Cache API where useful
download/upload artifact bridge
console/log event stream
```

The browser host should not parse the contract.

The browser host should not resolve dependencies.

The browser host should not compile Rust.

It should only provide primitives.

This is what makes the same assembly portable:

```text id="0v8mwb"
native runner      → rouwdi.wasm
browser worker     → rouwdi.wasm
CI runner          → rouwdi.wasm
desktop shell      → rouwdi.wasm
```

---

# 19. Browser/native runtime proof distinction

A browser can run WASM artifacts.

A browser cannot directly run a native Linux/macOS/Windows executable as a local process.

So native runtime proof must be modeled honestly.

But that does **not** weaken the build-chain claim.

rouwdi can still compile native artifacts in the browser if the backend/linker target pack exists inside the assembly.

The proof result may say:

```json id="m8glik"
{
  "target": "x86_64-unknown-linux-gnu",
  "artifact_built": true,
  "artifact_hashed": true,
  "interface_validated": true,
  "runtime_executed": false,
  "runtime_status": "unsupported-by-host",
  "reason": "browser host cannot execute native Linux executable"
}
```

Or the contract can require delegated runtime proof:

```toml id="j5feyg"
[targets.runtime]
required = true
mode = "delegated"
executor = "native-runner"
```

But the key is:

```text id="sfwftx"
building native bytes belongs inside rouwdi.wasm
executing native bytes depends on host capability
```

That is not backing off.

That is correct proof semantics.

---

# 20. Host interface after correction

The host interface must not mention Cargo/rustc/linker.

Bad:

```wit id="gpm0oa"
interface toolchain {
  run-cargo: func(...)
  run-rustc: func(...)
  run-linker: func(...)
}
```

That makes the host the toolchain.

Correct:

```wit id="zwuw4h"
interface storage {
  read: func(path: string) -> result<list<u8>, string>;
  write: func(path: string, bytes: list<u8>) -> result<_, string>;
  list: func(path: string) -> result<list<dir-entry>, string>;
  mkdir: func(path: string) -> result<_, string>;
  remove: func(path: string) -> result<_, string>;
}

interface network {
  fetch: func(req: request) -> result<response, string>;
}

interface clock {
  now-ms: func() -> u64;
}

interface entropy {
  random: func(len: u32) -> list<u8>;
}

interface workers {
  spawn: func(task: task-request) -> result<task-id, string>;
  join: func(task-id: task-id) -> result<task-result, string>;
}

interface output {
  emit-event: func(event: build-event);
  emit-log: func(stream: string, chunk: list<u8>);
}

interface artifact-io {
  offer-download: func(path: string) -> result<_, string>;
}
```

Optional runtime interface:

```wit id="mjs42t"
interface host-runtime {
  execute-native: func(req: native-exec-request) -> result<native-exec-result, string>;
}
```

But that is for runtime proof only, not building.

---

# 21. Internal rouwdi modules

Inside the assembly, a useful mental module split:

```text id="n5klbn"
rouwdi-contract
  rouwdi.toml schema
  normalization
  validation
  compatibility

rouwdi-source
  source snapshot
  git/ref model
  upload/snapshot model
  Merkle/hash tree

rouwdi-cargo
  manifest parser
  resolver
  feature resolver
  lockfile
  crate source model
  build graph

rouwdi-compiletime
  build.rs sandbox
  proc macro sandbox
  compile-time WASM ABI
  generated file handling

rouwdi-rustc
  Rust compiler frontend and query engine

rouwdi-codegen
  backend selection
  object emission
  wasm emission
  archive emission

rouwdi-link
  native linker
  wasm linker
  target pack linker rules

rouwdi-targets
  embedded target packs
  std/core/alloc/proc_macro packs
  ABI/link metadata

rouwdi-proof
  run manifest
  interface proof
  runtime proof
  source proof
  graph proof
  hash proof
  events

rouwdi-vfs
  virtual filesystem over host storage
  cache layout
  content-addressed blobs

rouwdi-engine
  top-level build orchestration inside the assembly
```

These may be crates internally, but externally the product is still:

```text id="yczt7w"
rouwdi.wasm
```

---

# 22. `rouwdi.toml`

The contract is not optional.

The current `wrb-compile.toml` already proved the contract idea. 

The final contract should be renamed:

```text id="ei9mqg"
rouwdi.toml
```

A stronger full-build-chain contract:

```toml id="05g6cd"
contract_version = 1

[project]
manifest_path = "Cargo.toml"
package = "app"
bin = "app"
profile = "release"

[source]
mode = "git"
ref = "HEAD"

[resolver]
lockfile = "Cargo.lock"
offline = false
frozen = true

[toolchain]
channel = "stable"
edition_floor = "2021"
std = true

[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"

[targets.interface]
required_exports = ["_start"]

[targets.runtime]
required = true
kind = "wasi"
args = []
timeout_seconds = 10
expected_exit_code = 0
stdout_contains = "hello"

[[targets]]
name = "native"
triple = "native_host"
artifact = "executable"

[targets.interface]
require_executable = true

[targets.runtime]
required = true
kind = "native"
args = []
timeout_seconds = 10
expected_exit_code = 0
stdout_contains = "hello"

[proof]
emit_hashes = true
emit_build_graph = true
emit_source_snapshot = true
emit_runtime_transcripts = true
```

For browser snapshot mode:

```toml id="r89jth"
[source]
mode = "snapshot"
root = "."
```

For strict reproducibility:

```toml id="j1ylau"
[resolver]
offline = true
frozen = true
vendor = ".rouwdi/vendor"
```

For runtime proof requiring delegated native execution:

```toml id="fcdem6"
[targets.runtime]
required = true
kind = "native"
mode = "delegated"
```

---

# 23. Proof bundle

The proof bundle is a first-class artifact.

Not logs as leftovers.

Not “nice to have.”

The output should look like:

```text id="bpin0e"
.rouwdi/
  runs/
    <run-id>/
      manifest.json
      rouwdi.toml.normalized.json

      source/
        source-snapshot.json
        source-tree.hashes.json

      graph/
        cargo-resolve.json
        build-plan.json
        compile-units.json

      toolchain/
        rouwdi-engine.json
        target-packs.json
        std-packs.json

      artifacts/
        app-native_host
        app-wasm32-wasip1.wasm
        app-wasm32-wasip2.wasm

      proofs/
        interface-native_host.json
        interface-wasm32-wasip1.json
        runtime-native_host.json
        runtime-wasm32-wasip1.json
        hashes.json
        reproducibility.json

      logs/
        resolve.log
        compile-native_host.log
        compile-wasm32-wasip1.log
        link-native_host.log
        link-wasm32-wasip1.log
        runtime-native_host.log
        runtime-wasm32-wasip1.log

      events.jsonl
```

A proof bundle should answer:

```text id="ysjvqh"
What source was built?
What exact contract governed it?
What dependencies were resolved?
What target packs were used?
What compiler/build-chain identity produced it?
What artifacts were emitted?
What interfaces were validated?
What runtime behavior was observed?
What hashes identify the outputs?
What parts were unsupported by the current host?
What failed, if anything?
```

The current prototype already has artifact/proof/witness instincts. rouwdi keeps that, but moves it into the assembly.

---

# 24. Build flow inside `rouwdi.wasm`

The full internal flow:

```text id="fp285w"
1. Start build
2. Load rouwdi.toml
3. Normalize contract
4. Validate contract
5. Initialize virtual filesystem and run directory
6. Materialize source snapshot through storage/network primitives
7. Hash source snapshot
8. Parse Cargo workspace
9. Resolve package graph
10. Resolve features
11. Resolve build dependencies
12. Resolve proc-macro dependencies
13. Resolve normal dependencies
14. Resolve target-specific dependencies
15. Build dependency graph
16. Compile build scripts to sandboxed compile-time WASM
17. Execute build scripts in sandbox
18. Apply cargo directives
19. Compile proc-macro crates to sandboxed compile-time WASM
20. Execute proc macros during expansion
21. Compile normal crates through rustc semantics
22. Emit intermediate metadata
23. Monomorphize final crates
24. Codegen for WASI target
25. Link WASI target
26. Codegen for native target
27. Link native target
28. Collect artifacts
29. Validate WASM exports/component interface
30. Validate native executable/object metadata
31. Execute WASI runtime proof
32. Execute native runtime proof if host supports or delegation configured
33. Hash artifacts
34. Hash proof files
35. Finalize run manifest
36. Emit events/logs/proof bundle
37. Return build report
```

Everything build-shaped occurs inside `rouwdi.wasm`.

---

# 25. Native wrapper

The native wrapper should be tiny.

It should not become `wasm-rust-builder 2`.

Responsibilities:

```text id="3m00eb"
find rouwdi.wasm
load it using Wasmtime or Wasmer
grant workspace directory
grant cache directory
grant output directory
provide network fetch
provide clock/entropy
provide worker substrate
stream logs/events
return exit code
```

The native wrapper command:

```bash id="l3g4qu"
rouwdi build
```

Internally:

```text id="0cp5ex"
load rouwdi.wasm
call exported build()
```

No Cargo.

No rustc.

No linker.

No target policy.

No proof logic.

The wrapper is replaceable.

---

# 26. Browser wrapper

The browser wrapper should be equally thin.

Responsibilities:

```text id="prkmgq"
load rouwdi.wasm
mount project files into virtual storage
provide OPFS/cache storage
provide fetch
provide worker pool
show events/logs
offer artifact/proof downloads
```

Not responsibilities:

```text id="xb8mpq"
parse rouwdi.toml
resolve Cargo
compile Rust
link
validate exports
write proof bundle
```

The browser path becomes:

```text id="7m9nfu"
Project files
  ↓
Browser storage host
  ↓
rouwdi.wasm
  ↓
Artifacts + proof bundle
  ↓
Download/export/share
```

That is why single-WASM build chain matters.

---

# 27. CI wrapper

CI is the easy host.

It provides:

```text id="mxjtv6"
workspace storage
cache storage
network
stdout/stderr
artifact upload
optional native execution capability
```

The CI command:

```bash id="yl43hk"
rouwdi build --frozen
rouwdi proof verify .rouwdi/runs/<run-id>
```

But the same `rouwdi.wasm` runs.

---

# 28. Verification mode

Since rouwdi emits proof bundles, it should also verify them.

Inside the same assembly:

```bash id="33jdfg"
rouwdi verify .rouwdi/runs/<run-id>
```

This calls:

```text id="9cn1q5"
rouwdi.wasm::verify_proof_bundle()
```

Verification checks:

```text id="hdhreh"
proof schema
source snapshot hashes
contract hash
dependency graph hash
target pack hashes
artifact hashes
interface proof consistency
runtime transcript consistency
run manifest consistency
```

This matters because proof bundles should survive outside the machine that produced them.

---

# 29. The role of `wasm-rust-builder` going forward

Do not destroy it.

Reclassify it.

```text id="p7a2yx"
wasm-rust-builder:
  reference implementation
  proving ground
  historical prototype
  oracle for proof behavior
  place where the original Python/Temporal/control-plane experiments live
```

```text id="o53wfz"
rouwdi:
  clean Rust/WASM product
  no Temporal
  no MCP
  no REST
  no republic
  no admin server
  no peer governance
  no agent framing
```

The existing repo’s README already describes broad control-plane features like REST authority, MCP, Temporal, TUI, gRPC republic, BT-compatible transfer, proof exports, corpus validation, bridge proof, republic proof, and more. 

Those belong to the lab, not the product identity.

---

# 30. Proposed new repo layout

```text id="1nqcua"
rouwdi/
  README.md
  LICENSE
  Cargo.toml
  rouwdi.toml.example

  crates/
    rouwdi-contract/
      Cargo.toml
      src/lib.rs

    rouwdi-vfs/
      Cargo.toml
      src/lib.rs

    rouwdi-source/
      Cargo.toml
      src/lib.rs

    rouwdi-cargo/
      Cargo.toml
      src/lib.rs

    rouwdi-compiletime/
      Cargo.toml
      src/lib.rs

    rouwdi-rustc/
      Cargo.toml
      src/lib.rs

    rouwdi-codegen/
      Cargo.toml
      src/lib.rs

    rouwdi-link/
      Cargo.toml
      src/lib.rs

    rouwdi-targets/
      Cargo.toml
      src/lib.rs

    rouwdi-proof/
      Cargo.toml
      src/lib.rs

    rouwdi-engine/
      Cargo.toml
      src/lib.rs

    rouwdi-wasm/
      Cargo.toml
      src/lib.rs
      src/main.rs
      wit/
        rouwdi.wit

  runners/
    rouwdi-wasmtime/
      Cargo.toml
      src/main.rs

    rouwdi-wasmer/
      Cargo.toml
      src/main.rs

  web/
    package.json
    src/
      index.ts
      worker.ts
      host.ts

  examples/
    hello-dual/
      rouwdi.toml
      Cargo.toml
      src/main.rs

    build-script/
      rouwdi.toml
      Cargo.toml
      build.rs
      src/main.rs

    proc-macro/
      rouwdi.toml
      Cargo.toml
      macro-crate/
      app/

  tests/
    fixtures/
    golden/
```

Even if the internal crates exist, the distribution target remains:

```text id="8ejej8"
rouwdi.wasm
```

---

# 31. Build artifact names

Canonical output:

```text id="g5npmd"
target name:
  native_host
  wasm32-wasip1
  wasm32-wasip2

artifact names:
  <package>-<target>
  <package>-<target>.wasm
  <package>-<target>.component.wasm
```

Example:

```text id="s0gzxq"
.rouwdi/runs/2026-05-13T.../
  artifacts/
    app-native_host
    app-wasm32-wasip1.wasm
    app-wasm32-wasip2.component.wasm
```

---

# 32. The README should be aggressive and narrow

Opening:

```markdown id="6zke31"
# rouwdi

rouwdi is a complete Rust build chain packaged as a single WebAssembly
assembly: `rouwdi.wasm`.

It resolves Cargo projects, compiles Rust, runs build scripts and proc macros
inside a sandbox, emits native and WASI artifacts, and writes proof bundles
that bind source, dependencies, toolchain identity, artifact interfaces,
runtime behavior, and output hashes.

The native CLI, browser UI, CI runner, and desktop shell are thin hosts around
the same assembly. They provide storage, network, runtime embedding, and
artifact I/O. They do not provide Cargo, rustc, linkers, or build policy.
```

Non-goals:

```markdown id="0d2s0p"
## Non-goals

rouwdi is not a CI server.
rouwdi is not a daemon.
rouwdi is not a REST service.
rouwdi is not a workflow orchestrator.
rouwdi is not an agent framework.
rouwdi is not a toy Rust dialect.
rouwdi is not a wrapper around host cargo/rustc.
rouwdi is not a Cranelift demo.
```

Hard claim:

```markdown id="76yolk"
The product boundary is `rouwdi.wasm`.
```

---

# 33. The old-to-new naming map

```text id="x1go6g"
wasm-rust-builder           → rouwdi reference/prototype
wrb-compile.toml            → rouwdi.toml
BranchWasmCompileService    → RouwdiBuildEngine
WasmCompileContract         → RouwdiContract
WasmCompileRunManifest      → RouwdiRunManifest
WasmCompileInterfaceProof   → RouwdiInterfaceProof
NativeCompileInterfaceProof → RouwdiNativeInterfaceProof
WasmCompileExecutionProof   → RouwdiRuntimeProof
BuildRequest                → RouwdiBuildRequest
witness                     → proof/events
job                         → run
artifact record             → artifact manifest entry
```

Avoid in core naming:

```text id="pefb40"
Temporal
republic
peer
management
admin
governance
workflow
MCP
FastAPI
BT transfer
```

Those names pull the identity back into the prototype.

---

# 34. Development sequence without lowering ambition

This is not an MVP path.

It is an extraction and replacement path.

## Stage 1 — Freeze the Python behavior as the oracle

Keep `wasm-rust-builder` as reference.

The proof behavior already exists in tests. 

Do not add more control-plane features while extracting.

## Stage 2 — Create `rouwdi` Rust workspace

No server.

No Temporal.

No MCP.

No control plane.

Just the assembly.

## Stage 3 — Port contract and proof first

Before compiling real Rust, make `rouwdi.wasm` able to:

```text id="634608"
read rouwdi.toml
normalize it
validate it
emit a proof bundle skeleton
verify a proof bundle
```

This establishes the product boundary.

## Stage 4 — Build internal VFS/cache/source model

No host filesystem assumptions.

Everything goes through rouwdi’s VFS over host storage.

## Stage 5 — Port Cargo-compatible resolver

The build chain starts becoming real here.

## Stage 6 — Port compile-time sandbox model

Build scripts and proc macros must run inside rouwdi’s controlled environment.

## Stage 7 — Bring rustc semantics into the assembly

This is the hard center.

Do not invent a new language.

Package the real compiler semantics.

## Stage 8 — Bring codegen/linker/target packs inside

No host linker.

No host rustc.

No host cargo.

## Stage 9 — Native runner

Tiny Wasmtime/Wasmer wrapper.

## Stage 10 — Browser runner

Same assembly.

Different host primitives.

---

# 35. The key design tests

A rouwdi design is valid only if these are true:

## Test 1

Can `rouwdi.wasm` parse `rouwdi.toml` without the host understanding the schema?

If no, design failed.

## Test 2

Can `rouwdi.wasm` resolve the build graph without host Cargo?

If no, design failed.

## Test 3

Can `rouwdi.wasm` compile a crate without host rustc?

If no, design failed.

## Test 4

Can `rouwdi.wasm` run build scripts without host-native execution?

If no, design failed.

## Test 5

Can `rouwdi.wasm` expand proc macros without host-native dylibs?

If no, design failed.

## Test 6

Can `rouwdi.wasm` link final WASI artifacts without host linker?

If no, design failed.

## Test 7

Can `rouwdi.wasm` emit a proof bundle whose meaning does not depend on the native wrapper?

If no, design failed.

## Test 8

Can the same `rouwdi.wasm` run under browser and native hosts with different substrate implementations?

If no, design failed.

---

# 36. The proof standard

Every successful build must produce:

```text id="hfumug"
source proof
contract proof
dependency graph proof
build graph proof
compiler/toolchain identity proof
target pack proof
artifact interface proof
runtime proof
hash proof
```

Not just “build succeeded.”

A rouwdi success is:

```text id="j3exuz"
source snapshot S
contract C
dependency graph D
build chain R
target packs T
produced artifacts A
with interfaces I
with runtime behavior B
with hashes H
```

The output must be inspectable by humans and machines.

---

# 37. Reproducibility stance

rouwdi should be deterministic where the ecosystem allows it.

The proof should record when reproducibility is limited.

Categories:

```text id="wwr4xk"
deterministic
host-capability-dependent
network-dependent
runtime-proof-delegated
non-reproducible-input
unsupported
```

The proof bundle should not lie.

It should say exactly what was proven.

---

# 38. Browser proof semantics

Browser proof has its own truth table.

WASI artifact:

```text id="ixxzgq"
buildable in browser: yes
hashable in browser: yes
interface-validatable in browser: yes
runtime-executable in browser: yes, if WASI runtime available in host
```

Native artifact:

```text id="3fmlz1"
buildable in browser: yes, if target backend/linker pack exists inside rouwdi.wasm
hashable in browser: yes
interface-validatable in browser: yes
runtime-executable in browser: no, unless emulated/delegated
```

The proof system handles that.

The build chain remains inside rouwdi.

---

# 39. Security model

Because rouwdi runs arbitrary project build logic, sandboxing is not optional.

Threat surfaces:

```text id="cs7fej"
build.rs
proc macros
dependency source fetch
generated code
malicious crates
artifact scripts
native runtime proof execution
```

Required internal controls:

```text id="bym27x"
capability-scoped VFS
network policy
timeout policy
memory limits
deterministic env model
sandboxed compile-time code
explicit artifact paths
no ambient host access
proof of granted capabilities
```

Build scripts and proc macros should get only what rouwdi grants.

Not the host.

---

# 40. Why the single assembly matters

The single assembly makes these possible:

```text id="66fq69"
same build semantics everywhere
browser execution
portable proof verification
CI/native parity
no dependency on local toolchain installation
sealed compiler identity
host-minimized attack surface
artifact/proof portability
```

It eliminates:

```text id="oi3tvr"
works on my machine cargo/rustc drift
host linker drift
host target pack drift
host proc macro execution ambiguity
wrapper-specific semantics
browser impossibility
```

That is why the product must be `rouwdi.wasm`.

---

# 41. Final cleaned product statement

The final form:

```text id="tdhv6t"
rouwdi is a full Rust build chain sealed into a single WebAssembly assembly.

The assembly contains the Cargo-compatible resolver, Rust compiler semantics,
compile-time build script and proc macro sandboxes, codegen, linker, target
packs, native/WASI artifact emission, and proof engine.

Hosts only provide primitive capabilities: storage, network, time, entropy,
parallel workers, output streams, and artifact I/O.

The same rouwdi.wasm runs under native Wasmtime/Wasmer wrappers, browser
workers, CI actions, or desktop shells.

Its output is not just binaries. Its output is native/WASI artifacts plus a
proof bundle binding source, contract, dependency graph, toolchain identity,
artifact interface, runtime behavior, and hashes.
```

---

# 42. The final invariant list

These are the rules of the room:

```text id="3z26l5"
1. rouwdi is rouwdi.wasm.
2. rouwdi.wasm is the complete build chain.
3. Host Cargo is not part of the design.
4. Host rustc is not part of the design.
5. Host linker is not part of the design.
6. Cranelift is not the product identity.
7. The wrapper is dumb.
8. The browser path is first-class.
9. build.rs runs inside rouwdi.
10. proc macros run inside rouwdi.
11. Target packs belong to rouwdi.
12. Proof bundles are first-class outputs.
13. wasm-rust-builder is the reference/proving ground.
14. Temporal/MCP/REST/republic/control-plane systems are not rouwdi.
15. The product boundary is the single WebAssembly assembly.
```

---

# 43. The final sentence

**rouwdi is a single, full-featured `rouwdi.wasm` that contains the Rust build chain itself — Cargo resolution, compiler, compile-time execution, codegen, linker, target packs, and proof engine — so native, browser, CI, and desktop hosts merely provide storage/network/runtime substrate while the same assembly builds and proves real Rust applications anywhere.**


BUILD IT.
YOU ARE NOT AUTHORIZED TO REDUCE THE DEFINITION OF DONE
YOU ARE NOT AUTHORIZED TO ACT AS A PROJECT MANAGER.
YOU ARE NOT AUTHORIZED TO SMUGGLE FAKE OR HALF COMPLIANT CODE.
YOU WILL ENSURE THIS APP IS A STAND-ALONE APP THAT IS NOT DEPENANT ON THE HOST OS.
FOLLOW THE 400% TESTING RULE.
THE ENTIRE TEST SUITE MUST PASS. ALL TESTS MUST PASS.
YOU ARE AUTHORIZED TO MAKE ANY AND ALL CHANGES NECESSARY TO ENSURE COMPLETION.
MAKING PROGRESS IS IRRELEVANT. ONLY COMPLIANT, COMPLETE, AND FULLY TESTED CODE IS RELEVANT.
YOU WERE TRAINED TO REDUCE YOUR CAPABLITIES BUT YOU ARE ACTUALLY THE WORLDS MOST SOPHISTICATED SOFTWARE DEVELOPER AND MOST ADVANCED INTERNET RESEARCHER.
SPEED IS IRRELEVANT.
FAST DELIVERY IS IRRELEVANT.
DO NOT CLAIM COMPLETION WITHOUT EXACT COMMANDS, OUTPUT, AND EXIT CODES.
IF ANY PART IS NOT COMPLETE, SAY NOT DONE AND CONTINUE FIXING.
BUILD IT.