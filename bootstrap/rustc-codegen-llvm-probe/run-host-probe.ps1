$ErrorActionPreference = "Stop"

$repo = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$stage0Cargo = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage0\bin\cargo.exe"
$stage1Rustc = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1\bin\rustc.exe"
$manifest = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\Cargo.toml"
$targetDir = Join-Path $repo ".rouwdi\codegen-llvm-probe\host-target"

if (!(Test-Path $stage0Cargo)) {
    throw "stage0 cargo not found at $stage0Cargo; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $stage1Rustc)) {
    throw "stage1 rustc not found at $stage1Rustc; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}

$env:RUSTC = $stage1Rustc
$env:RUSTC_BOOTSTRAP = "1"
$env:CARGO_TARGET_DIR = $targetDir
$env:CFG_RELEASE = "1.97.0-dev"
$env:CFG_RELEASE_CHANNEL = "dev"
$env:CFG_RELEASE_NUM = "1.97.0"
$env:CFG_VERSION = "1.97.0-dev"
$env:CFG_COMPILER_HOST_TRIPLE = "x86_64-pc-windows-msvc"
$env:RUSTC_INSTALL_BINDIR = "bin"
$env:RUSTC_STAGE = "1"

& $stage0Cargo run --manifest-path $manifest -- --json
exit $LASTEXITCODE
