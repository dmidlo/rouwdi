$ErrorActionPreference = "Stop"

$repo = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$stage0Cargo = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage0\bin\cargo.exe"
$stage1Rustc = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1\bin\rustc.exe"
$wasiSdk = Join-Path $repo ".rouwdi\tools\wasi-sdk\wasi-sdk-33.0-x86_64-windows"
$wasiClang = Join-Path $wasiSdk "bin\wasm32-wasip1-clang.exe"
$wasiClangxx = Join-Path $wasiSdk "bin\wasm32-wasip1-clang++.exe"
$wasiAr = Join-Path $wasiSdk "bin\llvm-ar.exe"
$wasiRanlib = Join-Path $wasiSdk "bin\llvm-ranlib.exe"
$wasiSysroot = Join-Path $wasiSdk "share\wasi-sysroot"
$targetDir = Join-Path $repo ".rouwdi\codegen-llvm-probe\wasm-target"

if (!(Test-Path $stage0Cargo)) {
    throw "stage0 cargo not found at $stage0Cargo; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $stage1Rustc)) {
    throw "stage1 rustc not found at $stage1Rustc; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $wasiClang)) {
    throw "WASI SDK clang not found at $wasiClang"
}

$env:RUSTC = $stage1Rustc
$env:RUSTC_BOOTSTRAP = "1"
$env:CARGO_TARGET_DIR = $targetDir
$env:CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS = "-Zunstable-options --cfg=bootstrap -C relocation-model=pic --sysroot $repo\third_party\rust\build\x86_64-pc-windows-msvc\stage1"
$env:CARGO_TARGET_WASM32_WASIP1_LINKER = $wasiClang
$env:CC_wasm32_wasip1 = $wasiClang
$env:CXX_wasm32_wasip1 = $wasiClangxx
$env:AR_wasm32_wasip1 = $wasiAr
$env:RANLIB_wasm32_wasip1 = $wasiRanlib
$env:WASI_SYSROOT = $wasiSysroot
$env:CFLAGS_wasm32_wasip1 = "--sysroot=$wasiSysroot"
$env:CXXFLAGS_wasm32_wasip1 = "--sysroot=$wasiSysroot"
$env:CFG_RELEASE = "1.97.0-dev"
$env:CFG_RELEASE_CHANNEL = "dev"
$env:CFG_RELEASE_NUM = "1.97.0"
$env:CFG_VERSION = "1.97.0-dev"
$env:CFG_COMPILER_HOST_TRIPLE = "x86_64-pc-windows-msvc"
$env:RUSTC_INSTALL_BINDIR = "bin"
$env:RUSTC_STAGE = "1"
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

& $stage0Cargo check --manifest-path (Join-Path $repo "third_party\rust\Cargo.toml") -p rustc_codegen_llvm --target wasm32-wasip1 --release --features rustc_codegen_llvm/check_only --message-format short
exit $LASTEXITCODE
