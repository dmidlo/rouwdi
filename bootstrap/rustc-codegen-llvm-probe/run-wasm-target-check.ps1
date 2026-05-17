$ErrorActionPreference = "Stop"

$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$stage0Cargo = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage0\bin\cargo.exe"
$stage1Rustc = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1\bin\rustc.exe"
$wasiSdk = Join-Path $repo ".rouwdi\tools\wasi-sdk\wasi-sdk-33.0-x86_64-windows"
$wasiClang = Join-Path $wasiSdk "bin\wasm32-wasip1-clang.exe"
$wasiClangxx = Join-Path $wasiSdk "bin\wasm32-wasip1-clang++.exe"
$wasiAr = Join-Path $wasiSdk "bin\llvm-ar.exe"
$wasiRanlib = Join-Path $wasiSdk "bin\llvm-ranlib.exe"
$wasiSysroot = Join-Path $wasiSdk "share\wasi-sysroot"
$wasiTargetLibDir = Join-Path $wasiSysroot "lib\wasm32-wasip1"
$wasiCxxLibDir = Join-Path $wasiSysroot "lib\wasm32-wasip1\noeh"
$shimIncludeDir = Join-Path $repo "bootstrap\wasi-sdk-shims\include"
$llvmConfig = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\target-llvm-config.cmd"
$targetLlvmLibDir = Join-Path $repo "third_party\rust\build\wasm32-wasip1\llvm\build\lib"
$targetDir = Join-Path $repo ".rouwdi\codegen-llvm-probe\wasm-target"
$reportDir = Join-Path $repo ".rouwdi\codegen-llvm-probe"
$reportPath = Join-Path $reportDir "wasm-target-report.json"
$checkLogPath = Join-Path $reportDir "wasm-target-check.log"
$payloadBuildLogPath = Join-Path $reportDir "wasm-backend-payload-build.log"
$probeManifest = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\Cargo.toml"
$wrapperBuildScript = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\build-target-llvm-wrapper.ps1"
$wrapperReportPath = Join-Path $reportDir "target-llvm-wrapper-report.json"
$supportShimSource = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\wasi-llvm-support-shims.c"
$supportShimDir = Join-Path $reportDir "target-llvm-support-shims"
$supportShimLibDir = Join-Path $supportShimDir "lib"
$supportShimObjDir = Join-Path $supportShimDir "obj"
$supportShimObject = Join-Path $supportShimObjDir "wasi-llvm-support-shims.o"
$supportShimArchive = Join-Path $supportShimLibDir "librouwdi-llvm-support-shims.a"
$supportShimCompileLogPath = Join-Path $supportShimDir "wasi-llvm-support-shims.compile.log"
$supportShimArchiveLogPath = Join-Path $supportShimDir "wasi-llvm-support-shims.archive.log"

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

& powershell -ExecutionPolicy Bypass -File $wrapperBuildScript
if ($LASTEXITCODE -ne 0) {
    throw "target llvm-wrapper build attempt failed with exit code $LASTEXITCODE"
}
$wrapperReport = Get-Content -Raw -LiteralPath $wrapperReportPath | ConvertFrom-Json
$wrapperLibPath = if ($wrapperReport.wrapper_archive_emitted -eq $true -and -not [string]::IsNullOrWhiteSpace([string]$wrapperReport.path)) {
    Join-Path $repo ([string]$wrapperReport.path)
} else {
    $null
}
$wrapperSearchFlag = if ($null -ne $wrapperLibPath -and (Test-Path -LiteralPath $wrapperLibPath -PathType Leaf)) {
    " -L native=$(Split-Path -Parent $wrapperLibPath)"
} else {
    ""
}

function Convert-ToForwardSlashPath {
    param([string]$Path)
    return $Path.Replace('\', '/')
}

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

$targetCompatFlagList = @(
    "--sysroot=$wasiSysroot",
    "-I$shimIncludeDir",
    "-D_WASI_EMULATED_SIGNAL",
    "-D_WASI_EMULATED_MMAN",
    "-D_WASI_EMULATED_PROCESS_CLOCKS",
    "-D__wasilibc_unmodified_upstream",
    "-DHAVE_SYS_MMAN_H=1",
    "-mllvm",
    "-wasm-enable-sjlj"
)
$targetCompatFlags = $targetCompatFlagList -join " "

New-Item -ItemType Directory -Force -Path $supportShimLibDir | Out-Null
New-Item -ItemType Directory -Force -Path $supportShimObjDir | Out-Null
$supportShimCompileResult = Invoke-CapturedNative -LogPath $supportShimCompileLogPath -Command {
    & $wasiClang @targetCompatFlagList -c $supportShimSource -o $supportShimObject
}
$supportShimArchiveResult = $null
if ($supportShimCompileResult.exit_code -eq 0) {
    $supportShimArchiveResult = Invoke-CapturedNative -LogPath $supportShimArchiveLogPath -Command {
        & $wasiAr crs $supportShimArchive $supportShimObject
    }
}
$supportShimBuilt = $supportShimCompileResult.exit_code -eq 0 -and
    $null -ne $supportShimArchiveResult -and
    $supportShimArchiveResult.exit_code -eq 0 -and
    (Test-Path -LiteralPath $supportShimArchive -PathType Leaf)
$supportShimHash = if ($supportShimBuilt) {
    (Get-FileHash -LiteralPath $supportShimArchive -Algorithm SHA256).Hash.ToLowerInvariant()
} else {
    $null
}
$supportShimSize = if ($supportShimBuilt) {
    [int64](Get-Item -LiteralPath $supportShimArchive).Length
} else {
    $null
}
$supportShimLinkFlag = if ($supportShimBuilt) {
    " -C link-arg=$supportShimArchive"
} else {
    ""
}

$env:RUSTC = $stage1Rustc
$env:RUSTC_BOOTSTRAP = "1"
$env:CFG_RELEASE = "1.97.0-dev"
$env:CFG_RELEASE_CHANNEL = "dev"
$env:CFG_VERSION = "1.97.0-dev"
$env:CARGO_TARGET_DIR = $targetDir
$targetLlvmSearchFlag = if (Test-Path -LiteralPath $targetLlvmLibDir -PathType Container) {
    " -L native=$targetLlvmLibDir"
} else {
    ""
}
$wasiTargetSearchFlag = if (Test-Path -LiteralPath $wasiTargetLibDir -PathType Container) {
    " -L native=$wasiTargetLibDir"
} else {
    ""
}
$wasiCxxSearchFlag = if (Test-Path -LiteralPath $wasiCxxLibDir -PathType Container) {
    " -L native=$wasiCxxLibDir"
} else {
    ""
}
$wasiEmulationLinkFlags = " -C link-arg=-L$wasiTargetLibDir -C link-arg=-L$wasiCxxLibDir -C link-arg=-lwasi-emulated-mman -C link-arg=-lwasi-emulated-process-clocks -C link-arg=-lwasi-emulated-signal -C link-arg=-lc++ -C link-arg=-lc++abi"
$env:CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS = "-Zunstable-options --cfg=bootstrap -C relocation-model=pic -C link-self-contained=no --sysroot $repo\third_party\rust\build\x86_64-pc-windows-msvc\stage1$wrapperSearchFlag$targetLlvmSearchFlag$wasiTargetSearchFlag$wasiCxxSearchFlag$supportShimLinkFlag$wasiEmulationLinkFlags"
$env:CARGO_TARGET_WASM32_WASIP1_LINKER = $wasiClang
$env:CC_wasm32_wasip1 = $wasiClang
$env:CXX_wasm32_wasip1 = $wasiClangxx
$env:AR_wasm32_wasip1 = $wasiAr
$env:RANLIB_wasm32_wasip1 = $wasiRanlib
$env:WASI_SYSROOT = $wasiSysroot
$env:CFLAGS_wasm32_wasip1 = $targetCompatFlags
$env:CXXFLAGS_wasm32_wasip1 = $targetCompatFlags
$env:LLVM_CONFIG = $llvmConfig
$env:REAL_LIBRARY_PATH_VAR = "PATH"
$env:REAL_LIBRARY_PATH = $env:PATH
$targetLlvmLibDirArg = Convert-ToForwardSlashPath $targetLlvmLibDir
$wasiTargetLibDirArg = Convert-ToForwardSlashPath $wasiTargetLibDir
$wasiCxxLibDirArg = Convert-ToForwardSlashPath $wasiCxxLibDir
$env:LLVM_LINKER_FLAGS = "-L$targetLlvmLibDirArg -L$wasiTargetLibDirArg -L$wasiCxxLibDirArg -lwasi-emulated-mman -lwasi-emulated-process-clocks -lwasi-emulated-signal"
$env:LLVM_USE_LIBCXX = "1"
Remove-Item Env:LLVM_LINK_SHARED -ErrorAction SilentlyContinue
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
        & $stage0Cargo build --manifest-path $probeManifest --target wasm32-wasip1 --release
    }
    $payloadBuildResult.lines | ForEach-Object { Write-Host $_ }
}

$payloadBuildExitCode = if ($null -ne $payloadBuildResult) { $payloadBuildResult.exit_code } else { $null }
$payloadBuildText = if ($null -ne $payloadBuildResult) { $payloadBuildResult.text } else { "" }
$payloadArtifact = if ($payloadBuildExitCode -eq 0) {
    Get-ChildItem -LiteralPath (Join-Path $targetDir "wasm32-wasip1\release\deps") -File -Filter "*.wasm" |
        Where-Object { $_.Name -like "rouwdi_rustc_codegen_llvm_probe-*.wasm" } |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
} else {
    $null
}
$payloadArtifactPath = if ($null -ne $payloadArtifact) {
    (Resolve-Path -LiteralPath $payloadArtifact.FullName).Path
} else {
    $null
}
$payloadArtifactRelativePath = if ($null -ne $payloadArtifactPath) {
    $repoPrefix = $repo.TrimEnd([char[]]"\/") + [System.IO.Path]::DirectorySeparatorChar
    if ($payloadArtifactPath.StartsWith($repoPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        $payloadArtifactPath.Substring($repoPrefix.Length).Replace("\", "/")
    } else {
        $payloadArtifactPath.Replace("\", "/")
    }
} else {
    $null
}
$payloadArtifactSha256 = if ($null -ne $payloadArtifactPath) {
    (Get-FileHash -LiteralPath $payloadArtifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
} else {
    $null
}
$payloadArtifactSizeBytes = if ($null -ne $payloadArtifact) {
    [int64]$payloadArtifact.Length
} else {
    $null
}
$enzymeBlocked = (($checkResult.text + "`n" + $payloadBuildText).ToLowerInvariant()).Contains("enzyme") -and (($checkResult.text + "`n" + $payloadBuildText).ToLowerInvariant()).Contains("libloading")
$undefinedSymbolMatches = [regex]::Matches($payloadBuildText, 'undefined symbol:\s*([A-Za-z_][A-Za-z0-9_]*)')
$duplicateSymbolMatches = [regex]::Matches($payloadBuildText, 'duplicate symbol:\s*(.+)')
$payloadUndefinedSymbols = @(
    $undefinedSymbolMatches |
        ForEach-Object { $_.Groups[1].Value } |
        Select-Object -Unique
)
$payloadDuplicateSymbols = @(
    $duplicateSymbolMatches |
        ForEach-Object { $_.Groups[1].Value.Trim() } |
        Select-Object -Unique
)
$payloadLlvmUndefinedSymbols = @($payloadUndefinedSymbols | Where-Object { $_.StartsWith("LLVM") })
$knownWasiSupportSymbols = @(
    "alarm",
    "dlclose",
    "dlerror",
    "dlopen",
    "dlsym",
    "dup2",
    "execv",
    "execve",
    "execvp",
    "fork",
    "getpid",
    "getrlimit",
    "getuid",
    "posix_madvise",
    "setsid",
    "setrlimit",
    "sigaction",
    "sigemptyset",
    "sigfillset",
    "sigprocmask"
)
$payloadWasiSupportUndefinedSymbols = @(
    $payloadUndefinedSymbols |
        Where-Object { $knownWasiSupportSymbols -contains $_ }
)
$payloadFirstUndefinedSymbol = if ($payloadUndefinedSymbols.Count -gt 0) { [string]$payloadUndefinedSymbols[0] } else { $null }
$payloadFirstDuplicateSymbol = if ($payloadDuplicateSymbols.Count -gt 0) { [string]$payloadDuplicateSymbols[0] } else { $null }
$payloadLinkerInvoked = $payloadBuildText.Contains("linking with") -or $payloadBuildText.Contains("wasm-ld:")
$payloadLinkerPathMatch = [regex]::Match($payloadBuildText, 'linking with `([^`]+)` failed')
$payloadLinkerPath = if ($payloadLinkerPathMatch.Success) { $payloadLinkerPathMatch.Groups[1].Value } else { $null }
$closureReport = $wrapperReport.target_llvm_library_closure
$closureBlockerKind = if ($null -ne $closureReport -and -not [string]::IsNullOrWhiteSpace([string]$closureReport.status) -and [string]$closureReport.status -ne "available") {
    [string]$closureReport.status
} elseif ($wrapperReport.target_llvm_library_closure_available -ne $true -and -not [string]::IsNullOrWhiteSpace([string]$wrapperReport.blocker_kind)) {
    [string]$wrapperReport.blocker_kind
} else {
    "wasm_codegen_payload_blocked_at_target_llvm_library_closure"
}
$closureBlockerReason = if ($null -ne $closureReport -and -not [string]::IsNullOrWhiteSpace([string]$closureReport.reason)) {
    [string]$closureReport.reason
} elseif (-not [string]::IsNullOrWhiteSpace([string]$wrapperReport.blocker_reason)) {
    [string]$wrapperReport.blocker_reason
} else {
    "The executable wasm32-wasip1 backend payload needs a target-compatible LLVM library closure."
}
$payloadBlockerKind = if ($payloadBuildExitCode -eq 0) {
    "none"
} elseif ($payloadBuildText.Contains("self-contained linker was requested") -and $payloadBuildText.Contains("wasn't found")) {
    "wasm_codegen_payload_blocked_at_missing_self_contained_linker"
} elseif ($payloadBuildText.Contains("could not find native static library") -and $payloadBuildText.Contains("llvm-wrapper")) {
    "wasm32_llvm_wrapper_static_library_missing"
} elseif ($payloadDuplicateSymbols.Count -gt 0 -and $payloadBuildText.Contains("librustc_llvm") -and $payloadBuildText.Contains("libLLVM")) {
    "wasm_codegen_payload_blocked_at_duplicate_llvm_archive_objects"
} elseif ($payloadWasiSupportUndefinedSymbols.Count -gt 0) {
    "wasm_codegen_payload_blocked_at_wasi_posix_support_symbols"
} elseif ($payloadLlvmUndefinedSymbols.Count -gt 0) {
    $closureBlockerKind
} elseif ($wrapperReport.wrapper_archive_emitted -eq $true -and $wrapperReport.target_llvm_library_closure_available -ne $true) {
    $closureBlockerKind
} elseif ($payloadBuildExitCode -ne $null) {
    "wasm_backend_payload_build_failed"
} else {
    "wasm_backend_payload_not_attempted"
}
$payloadStatus = if ($payloadBuildExitCode -eq 0) {
    "wasm_codegen_payload_linked"
} elseif ($payloadBlockerKind -eq $closureBlockerKind) {
    "rustc_codegen_llvm_backend_payload_blocked_at_$closureBlockerKind"
} elseif ($payloadBlockerKind -eq "wasm_codegen_payload_blocked_at_missing_self_contained_linker") {
    "rustc_codegen_llvm_backend_payload_blocked_at_missing_self_contained_linker"
} elseif ($payloadBlockerKind -eq "wasm32_llvm_wrapper_static_library_missing") {
    "rustc_codegen_llvm_backend_payload_blocked_at_wasm32_llvm_wrapper_static_library_missing"
} elseif ($payloadBlockerKind -eq "wasm_codegen_payload_blocked_at_duplicate_llvm_archive_objects") {
    "rustc_codegen_llvm_backend_payload_blocked_at_duplicate_llvm_archive_objects"
} elseif ($payloadBlockerKind -eq "wasm_codegen_payload_blocked_at_wasi_posix_support_symbols") {
    "rustc_codegen_llvm_backend_payload_blocked_at_wasi_posix_support_symbols"
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
    executable_backend_payload_linked = ($payloadBuildExitCode -eq 0)
    backend_payload_artifact_path = $payloadArtifactRelativePath
    backend_payload_artifact_sha256 = $payloadArtifactSha256
    backend_payload_artifact_size_bytes = $payloadArtifactSizeBytes
    embedded_backend_payload_executed = $false
    backend_payload_final_link_invoked = (($payloadBuildExitCode -eq 0) -or $payloadLinkerInvoked)
    backend_payload_linker = if ($payloadBuildExitCode -eq 0) { $wasiClang } else { $payloadLinkerPath }
    backend_payload_undefined_symbols = @($payloadUndefinedSymbols)
    backend_payload_llvm_undefined_symbols = @($payloadLlvmUndefinedSymbols)
    backend_payload_wasi_support_undefined_symbols = @($payloadWasiSupportUndefinedSymbols)
    backend_payload_first_undefined_symbol = $payloadFirstUndefinedSymbol
    backend_payload_duplicate_symbols = @($payloadDuplicateSymbols)
    backend_payload_first_duplicate_symbol = $payloadFirstDuplicateSymbol
    backend_execution_status = $payloadStatus
    backend_constructed_in_payload = $false
    llvm_module_setup_invoked_in_payload = $false
    target_machine_setup_invoked_in_payload = $false
    blocker_kind = $payloadBlockerKind
    blocker_reason = if ($payloadBlockerKind -eq "wasm32_llvm_wrapper_static_library_missing") {
        "The wasm32-wasip1 check-only route still exits 0 after the Enzyme/libloading blocker was isolated, but producing an executable codegen backend payload now reaches the exact next target-loadable blocker: rustc_codegen_llvm still requires a wasm32-wasip1 target-loadable llvm-wrapper static library at final link."
    } elseif ($payloadBlockerKind -eq $closureBlockerKind) {
        "The wasm32-wasip1 llvm-wrapper archive is emitted as target object code and the executable backend payload reaches WASI clang/wasm-ld final link, but rustc_codegen_llvm still has unresolved LLVM C API symbols such as $payloadFirstUndefinedSymbol. The missing component is a wasm32-wasip1 target-compatible LLVM library closure. The target LLVM closure build was attempted and is blocked at ${closureBlockerKind}: $closureBlockerReason Native CI LLVM libraries are host evidence only."
    } elseif ($payloadBlockerKind -eq "wasm_codegen_payload_blocked_at_missing_self_contained_linker") {
        "The executable wasm32-wasip1 backend payload build reached final link but rustc requested a self-contained linker that is absent from the stage1 wasm32-wasip1 sysroot. The route must pass -C link-self-contained=no with the target WASI clang linker, or provision a target-loadable linker in the sysroot."
    } elseif ($payloadBlockerKind -eq "wasm_codegen_payload_blocked_at_duplicate_llvm_archive_objects") {
        "The executable wasm32-wasip1 backend payload build reached WASI clang/wasm-ld final link with a target-loadable llvm-wrapper and target-compatible LLVM archive closure, but LLVM object files are currently linked twice. rustc_llvm has already bundled LLVM objects into librustc_llvm.rlib, and the payload link also saw standalone libLLVM archives; first duplicate symbol: $payloadFirstDuplicateSymbol."
    } elseif ($payloadBlockerKind -eq "wasm_codegen_payload_blocked_at_wasi_posix_support_symbols") {
        "The executable wasm32-wasip1 backend payload build reached WASI clang/wasm-ld final link with a target-loadable llvm-wrapper and target-compatible LLVM archive closure, then blocked on LLVM Support references to POSIX dynamic-loading/process symbols that WASI does not provide. Missing symbols: $(@($payloadWasiSupportUndefinedSymbols) -join ', '). The rouwdi target support shim archive was built=$supportShimBuilt and linked as target object code; inspect the payload log for remaining unresolved symbols."
    } elseif ($payloadBuildExitCode -eq 0) {
        "none"
    } else {
        "The wasm32-wasip1 backend payload build failed; inspect .rouwdi/codegen-llvm-probe/wasm-backend-payload-build.log for exact compiler output."
    }
    enzyme_libloading_blocker_present = $enzymeBlocked
    check_log_path = ".rouwdi/codegen-llvm-probe/wasm-target-check.log"
    payload_build_log_path = ".rouwdi/codegen-llvm-probe/wasm-backend-payload-build.log"
    llvm_wrapper_target = $wrapperReport.llvm_wrapper_target
    llvm_wrapper_artifact_kind = $wrapperReport.artifact_kind
    llvm_wrapper_path = $wrapperReport.path
    llvm_wrapper_sha256 = $wrapperReport.sha256
    llvm_wrapper_size_bytes = $wrapperReport.size_bytes
    llvm_wrapper_built_by = $wrapperReport.built_by
    llvm_wrapper_linked_into = $wrapperReport.linked_into
    llvm_wrapper_target_loadable = $wrapperReport.target_loadable
    llvm_wrapper_blocker_kind = $wrapperReport.blocker_kind
    llvm_wrapper_blocker_reason = $wrapperReport.blocker_reason
    llvm_support_shims_target = "wasm32-wasip1"
    llvm_support_shims_artifact_kind = "staticlib"
    llvm_support_shims_path = if ($supportShimBuilt) { ".rouwdi/codegen-llvm-probe/target-llvm-support-shims/lib/librouwdi-llvm-support-shims.a" } else { $null }
    llvm_support_shims_sha256 = $supportShimHash
    llvm_support_shims_size_bytes = $supportShimSize
    llvm_support_shims_built_by = "bootstrap/rustc-codegen-llvm-probe/run-wasm-target-check.ps1"
    llvm_support_shims_target_loadable = $supportShimBuilt
    llvm_support_shims_compile_exit_code = $supportShimCompileResult.exit_code
    llvm_support_shims_archive_exit_code = if ($null -ne $supportShimArchiveResult) { $supportShimArchiveResult.exit_code } else { $null }
    llvm_support_shims_functions = @($knownWasiSupportSymbols)
    target_llvm_library_closure_available = $wrapperReport.target_llvm_library_closure_available
    target_llvm_library_closure_status = $wrapperReport.target_llvm_library_closure.status
    target_llvm_library_closure_blocker_kind = $closureBlockerKind
    target_llvm_library_closure_blocker_reason = $closureBlockerReason
    target_llvm_library_closure_report_path = $wrapperReport.target_llvm_library_closure.report_path
    target_llvm_library_closure_log_path = $wrapperReport.target_llvm_library_closure.log_path
    target_llvm_library_closure_build_exit_code = $wrapperReport.target_llvm_library_closure.build_exit_code
    target_llvm_library_closure_first_error = $wrapperReport.target_llvm_library_closure.first_error
}
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $reportPath -Encoding utf8
Write-Host "wasm target report: $reportPath"

exit $checkResult.exit_code
