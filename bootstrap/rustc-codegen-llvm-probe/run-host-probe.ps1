$ErrorActionPreference = "Stop"

$repo = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$stage0Cargo = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage0\bin\cargo.exe"
$stage1Rustc = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1\bin\rustc.exe"
$manifest = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\Cargo.toml"
$targetDir = Join-Path $repo ".rouwdi\codegen-llvm-probe\host-target"
$llvmConfig = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\ci-llvm\bin\llvm-config.exe"
$wrapperCandidates = @(
    (Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1-rustc\x86_64-pc-windows-msvc\release\build\rustc_llvm-0d4f61ce596f94b4\out"),
    (Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1-rustc\x86_64-pc-windows-msvc\release\build\rustc_llvm-df1e4c80cc0afe5c\out")
)

if (!(Test-Path $stage0Cargo)) {
    throw "stage0 cargo not found at $stage0Cargo; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $stage1Rustc)) {
    throw "stage1 rustc not found at $stage1Rustc; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $llvmConfig)) {
    throw "llvm-config not found at $llvmConfig; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
$wrapperDir = $wrapperCandidates | Where-Object { Test-Path (Join-Path $_ "llvm-wrapper.lib") } | Select-Object -First 1
if (!$wrapperDir) {
    throw "stage1 llvm-wrapper.lib not found under stage1-rustc build output"
}
$llvmLibDir = (& $llvmConfig --libdir).Trim()
if (!(Test-Path $llvmLibDir)) {
    throw "LLVM library directory from llvm-config not found: $llvmLibDir"
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
$env:CARGO_INCREMENTAL = "0"
$env:CARGO_PROFILE_DEV_DEBUG = "0"
Write-Host "stage1 llvm-wrapper.lib: $(Join-Path $wrapperDir "llvm-wrapper.lib")"
Write-Host "candidate CI LLVM lib directory: $llvmLibDir"
$env:RUSTFLAGS = "-L native=$wrapperDir -l static=llvm-wrapper"

& $stage0Cargo run --manifest-path $manifest -- --json
exit $LASTEXITCODE
