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
$reportDir = Join-Path $repo ".rouwdi\codegen-llvm-probe"
$reportPath = Join-Path $reportDir "wasm-target-report.json"
$checkLogPath = Join-Path $reportDir "wasm-target-check.log"
$payloadBuildLogPath = Join-Path $reportDir "wasm-backend-payload-build.log"
$probeManifest = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\Cargo.toml"

if (!(Test-Path $stage0Cargo)) {
    throw "stage0 cargo not found at $stage0Cargo; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $stage1Rustc)) {
    throw "stage1 rustc not found at $stage1Rustc; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $wasiClang)) {
    throw "WASI SDK clang not found at $wasiClang"
}
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

function Invoke-CapturedNative {
    param(
        [string]$LogPath,
        [scriptblock]$Command
    )

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & $Command 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }

    $lines = @($output | ForEach-Object { $_.ToString() })
    $lines | Set-Content -LiteralPath $LogPath -Encoding utf8
    return [ordered]@{
        exit_code = $exitCode
        lines = $lines
        text = ($lines -join "`n")
    }
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

$checkResult = Invoke-CapturedNative -LogPath $checkLogPath -Command {
    & $stage0Cargo check --manifest-path (Join-Path $repo "third_party\rust\Cargo.toml") -p rustc_codegen_llvm --target wasm32-wasip1 --release --features rustc_codegen_llvm/check_only --message-format short
}
$checkResult.lines | ForEach-Object { Write-Host $_ }

$payloadBuildResult = $null
if ($checkResult.exit_code -eq 0) {
    $payloadBuildResult = Invoke-CapturedNative -LogPath $payloadBuildLogPath -Command {
        & $stage0Cargo build --manifest-path $probeManifest --target wasm32-wasip1 --release --message-format short
    }
    $payloadBuildResult.lines | ForEach-Object { Write-Host $_ }
}

$payloadBuildExitCode = if ($null -ne $payloadBuildResult) { $payloadBuildResult.exit_code } else { $null }
$payloadBuildText = if ($null -ne $payloadBuildResult) { $payloadBuildResult.text } else { "" }
$enzymeBlocked = (($checkResult.text + "`n" + $payloadBuildText).ToLowerInvariant()).Contains("enzyme") -and (($checkResult.text + "`n" + $payloadBuildText).ToLowerInvariant()).Contains("libloading")
$payloadBlockerKind = if ($payloadBuildExitCode -eq 0) {
    "none"
} elseif ($payloadBuildText.Contains("could not find native static library") -and $payloadBuildText.Contains("llvm-wrapper")) {
    "wasm32_llvm_wrapper_static_library_missing"
} elseif ($payloadBuildExitCode -ne $null) {
    "wasm_backend_payload_build_failed"
} else {
    "wasm_backend_payload_not_attempted"
}
$payloadStatus = if ($payloadBuildExitCode -eq 0) {
    "rustc_codegen_llvm_backend_constructed_in_payload"
} elseif ($payloadBlockerKind -eq "wasm32_llvm_wrapper_static_library_missing") {
    "rustc_codegen_llvm_backend_payload_blocked_at_wasm32_llvm_wrapper_static_library_missing"
} else {
    "rustc_codegen_llvm_backend_payload_blocked_at_$payloadBlockerKind"
}

$report = [ordered]@{
    schema_version = 1
    probe_name = "rustc_codegen_llvm_wasm_target_route"
    check_only_status = if ($checkResult.exit_code -eq 0) { "rustc_codegen_llvm_target_loadable_check_only" } else { "rustc_codegen_llvm_target_loadable_check_failed" }
    check_only_exit_code = $checkResult.exit_code
    check_only_command = "cargo check -p rustc_codegen_llvm --target wasm32-wasip1 --release --features rustc_codegen_llvm/check_only"
    backend_payload_build_attempted = ($null -ne $payloadBuildResult)
    backend_payload_build_command = "cargo build --manifest-path bootstrap/rustc-codegen-llvm-probe/Cargo.toml --target wasm32-wasip1 --release"
    backend_payload_build_exit_code = $payloadBuildExitCode
    backend_execution_status = $payloadStatus
    backend_constructed_in_payload = ($payloadBuildExitCode -eq 0)
    llvm_module_setup_invoked_in_payload = $false
    target_machine_setup_invoked_in_payload = $false
    blocker_kind = $payloadBlockerKind
    blocker_reason = if ($payloadBlockerKind -eq "wasm32_llvm_wrapper_static_library_missing") {
        "The wasm32-wasip1 check-only route still exits 0 after the Enzyme/libloading blocker was isolated, but producing an executable codegen backend payload now reaches the exact next target-loadable blocker: rustc_codegen_llvm still requires a wasm32-wasip1 target-loadable llvm-wrapper static library at final link."
    } elseif ($payloadBuildExitCode -eq 0) {
        "none"
    } else {
        "The wasm32-wasip1 backend payload build failed; inspect .rouwdi/codegen-llvm-probe/wasm-backend-payload-build.log for exact compiler output."
    }
    enzyme_libloading_blocker_present = $enzymeBlocked
    check_log_path = ".rouwdi/codegen-llvm-probe/wasm-target-check.log"
    payload_build_log_path = ".rouwdi/codegen-llvm-probe/wasm-backend-payload-build.log"
}
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $reportPath -Encoding utf8
Write-Host "wasm target report: $reportPath"

exit $checkResult.exit_code
