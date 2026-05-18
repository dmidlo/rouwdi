$ErrorActionPreference = "Stop"

$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$reportDir = Join-Path $repo ".rouwdi\codegen-llvm-probe"
$reportPath = Join-Path $reportDir "target-llvm-closure-report.json"
$configureLogPath = Join-Path $reportDir "target-llvm-closure-configure.log"
$buildLogPath = Join-Path $reportDir "target-llvm-closure-build.log"
$llvmSourceDir = Join-Path $repo "third_party\rust\src\llvm-project\llvm"
$llvmBuildDir = Join-Path $repo "third_party\rust\build\wasm32-wasip1\llvm\build"
$cmake = Join-Path $repo ".rouwdi\tools\cmake\cmake-4.3.2-windows-x86_64\bin\cmake.exe"
$ninja = Join-Path $repo ".rouwdi\tools\ninja\ninja-1.13.2-win\ninja.exe"
$wasiSdk = Join-Path $repo ".rouwdi\tools\wasi-sdk\wasi-sdk-33.0-x86_64-windows"
$wasiClang = Join-Path $wasiSdk "bin\wasm32-wasip1-clang.exe"
$wasiClangxx = Join-Path $wasiSdk "bin\wasm32-wasip1-clang++.exe"
$shimIncludeDir = Join-Path $repo "bootstrap\wasi-sdk-shims\include"
$shimEndianHeader = Join-Path $shimIncludeDir "machine\endian.h"
$targetTriple = "wasm32-wasip1"
$requiredComponents = @(
    "LLVMWebAssemblyDisassembler",
    "LLVMMCDisassembler",
    "LLVMWebAssemblyAsmParser",
    "LLVMWebAssemblyCodeGen",
    "LLVMWebAssemblyUtils",
    "LLVMWebAssemblyDesc",
    "LLVMWebAssemblyInfo",
    "LLVMAsmPrinter",
    "LLVMCoverage",
    "LLVMLTO",
    "LLVMPlugins",
    "LLVMPasses",
    "LLVMIRPrinter",
    "LLVMHipStdPar",
    "LLVMCoroutines",
    "LLVMGlobalISel",
    "LLVMSelectionDAG",
    "LLVMCFGuard",
    "LLVMExtensions",
    "LLVMCodeGen",
    "LLVMTarget",
    "LLVMObjCARCOpts",
    "LLVMCodeGenTypes",
    "LLVMCGData",
    "LLVMipo",
    "LLVMInstrumentation",
    "LLVMVectorize",
    "LLVMSandboxIR",
    "LLVMLinker",
    "LLVMFrontendOpenMP",
    "LLVMFrontendDirective",
    "LLVMFrontendAtomic",
    "LLVMFrontendOffloading",
    "LLVMObjectYAML",
    "LLVMScalarOpts",
    "LLVMInstCombine",
    "LLVMBitWriter",
    "LLVMAggressiveInstCombine",
    "LLVMTransformUtils",
    "LLVMAnalysis",
    "LLVMProfileData",
    "LLVMSymbolize",
    "LLVMDebugInfoBTF",
    "LLVMDebugInfoPDB",
    "LLVMDebugInfoMSF",
    "LLVMDebugInfoCodeView",
    "LLVMDebugInfoGSYM",
    "LLVMDebugInfoDWARF",
    "LLVMObject",
    "LLVMTextAPI",
    "LLVMMCParser",
    "LLVMIRReader",
    "LLVMAsmParser",
    "LLVMMC",
    "LLVMDebugInfoDWARFLowLevel",
    "LLVMBitReader",
    "LLVMFrontendHLSL",
    "LLVMCore",
    "LLVMRemarks",
    "LLVMBitstreamReader",
    "LLVMBinaryFormat",
    "LLVMTargetParser",
    "LLVMOption",
    "LLVMSupport",
    "LLVMDemangle",
    "lldCommon",
    "lldWasm"
)

function To-RepoRelativePath {
    param([string]$Path)

    $resolved = if (Test-Path -LiteralPath $Path) {
        (Resolve-Path -LiteralPath $Path).Path
    } else {
        [System.IO.Path]::GetFullPath($Path)
    }
    $root = $repo.TrimEnd([char[]]"\/")
    $rootPrefix = $root + [System.IO.Path]::DirectorySeparatorChar
    if ($resolved.Equals($root, [System.StringComparison]::OrdinalIgnoreCase)) {
        return "."
    }
    if ($resolved.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $resolved.Substring($rootPrefix.Length).Replace("\", "/")
    }
    return $resolved.Replace("\", "/")
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
    [System.IO.File]::WriteAllLines($LogPath, [string[]]$lines)
    return [ordered]@{
        exit_code = $exitCode
        lines = $lines
        text = ($lines -join "`n")
        log_path = To-RepoRelativePath -Path $LogPath
    }
}

function Get-ArchiveIdentities {
    param([string[]]$Names)

    $archives = New-Object System.Collections.Generic.List[object]
    foreach ($name in $Names) {
        $matches = @(
            Get-ChildItem -LiteralPath $llvmBuildDir -Recurse -File -ErrorAction SilentlyContinue |
                Where-Object {
                    $_.Name -eq "lib$name.a" -or
                    $_.Name -eq "$name.lib" -or
                    $_.Name -eq "LLVM$name.lib" -or
                    $_.Name -eq "libLLVM$name.a"
                }
        )
        foreach ($item in $matches) {
            $hash = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            $archives.Add([ordered]@{
                component = $name
                path = To-RepoRelativePath -Path $item.FullName
                sha256 = $hash
                size_bytes = [int64]$item.Length
                target_compatible = $true
            })
        }
    }
    return @($archives.ToArray())
}

function New-Report {
    param(
        [bool]$PrerequisitesPresent,
        [object]$ConfigureResult,
        [object]$BuildResult,
        [object[]]$TargetArchives,
        [string]$BlockerKind,
        [string]$BlockerReason,
        [string]$FirstError
    )

    $builtComponentSet = New-Object System.Collections.Generic.HashSet[string]
    foreach ($archive in @($TargetArchives)) {
        [void]$builtComponentSet.Add([string]$archive.component)
    }
    $allRequiredArchivesPresent = $true
    foreach ($requiredComponent in $requiredComponents) {
        if (-not $builtComponentSet.Contains($requiredComponent)) {
            $allRequiredArchivesPresent = $false
            break
        }
    }
    $closureAvailable = $PrerequisitesPresent -and
        $null -ne $BuildResult -and
        $BuildResult.exit_code -eq 0 -and
        $allRequiredArchivesPresent

    return [ordered]@{
        schema_version = 1
        probe_name = "rustc_codegen_llvm_target_llvm_library_closure"
        llvm_wrapper_target = $targetTriple
        target_triple = $targetTriple
        artifact_kind = "staticlib-closure"
        build_attempted = $PrerequisitesPresent
        configure_attempted = $PrerequisitesPresent
        configure_exit_code = if ($null -ne $ConfigureResult) { $ConfigureResult.exit_code } else { $null }
        build_exit_code = if ($null -ne $BuildResult) { $BuildResult.exit_code } else { $null }
        configure_command = "cmake -S third_party/rust/src/llvm-project/llvm -B third_party/rust/build/wasm32-wasip1/llvm/build -DCMAKE_C_FLAGS=<wasi llvm flags> -DCMAKE_CXX_FLAGS=<wasi llvm flags> -DLLVM_HOST_TRIPLE=wasm32-wasip1 -DLLVM_DEFAULT_TARGET_TRIPLE=wasm32-wasip1 -DLLVM_TARGETS_TO_BUILD=WebAssembly -DLLVM_ENABLE_CRASH_OVERRIDES=OFF -DLLVM_ENABLE_THREADS=OFF"
        build_command = "ninja -C third_party/rust/build/wasm32-wasip1/llvm/build $($requiredComponents -join ' ')"
        cmake_path = if (Test-Path -LiteralPath $cmake) { To-RepoRelativePath -Path $cmake } else { To-RepoRelativePath -Path $cmake }
        ninja_path = if (Test-Path -LiteralPath $ninja) { To-RepoRelativePath -Path $ninja } else { To-RepoRelativePath -Path $ninja }
        wasi_clang = if (Test-Path -LiteralPath $wasiClang) { To-RepoRelativePath -Path $wasiClang } else { To-RepoRelativePath -Path $wasiClang }
        wasi_clangxx = if (Test-Path -LiteralPath $wasiClangxx) { To-RepoRelativePath -Path $wasiClangxx } else { To-RepoRelativePath -Path $wasiClangxx }
        shim_include_path = if (Test-Path -LiteralPath $shimIncludeDir) { To-RepoRelativePath -Path $shimIncludeDir } else { To-RepoRelativePath -Path $shimIncludeDir }
        shim_machine_endian_header = if (Test-Path -LiteralPath $shimEndianHeader) { To-RepoRelativePath -Path $shimEndianHeader } else { To-RepoRelativePath -Path $shimEndianHeader }
        configured_flags = @(
            "--target=wasm32-wasip1",
            "-ffunction-sections",
            "-fdata-sections",
            "-fno-exceptions",
            "-DLLVM_ON_UNIX",
            "-D_WASI_EMULATED_SIGNAL",
            "-D_WASI_EMULATED_MMAN",
            "-D_WASI_EMULATED_PROCESS_CLOCKS",
            "-D__wasilibc_unmodified_upstream",
            "-DHAVE_SYS_MMAN_H=1",
            "-mllvm -wasm-enable-sjlj",
            "-Ibootstrap/wasi-sdk-shims/include",
            "-DLLVM_HOST_TRIPLE=wasm32-wasip1",
            "-DLLVM_DEFAULT_TARGET_TRIPLE=wasm32-wasip1",
            "-DLLVM_TARGETS_TO_BUILD=WebAssembly",
            "-DLLVM_ENABLE_CRASH_OVERRIDES=OFF",
            "-DLLVM_ENABLE_THREADS=OFF"
        )
        required_components = @($requiredComponents)
        first_build_target = "required LLVM closure components"
        built_components = @($TargetArchives | ForEach-Object { $_.component } | Select-Object -Unique)
        target_compatible_archives = @($TargetArchives)
        closure_available = $closureAvailable
        target_loadable = $closureAvailable
        host_ci_llvm_reused_as_target = $false
        native_ci_llvm_libraries_usable_for_product = $false
        log_path = if ($null -ne $BuildResult) { $BuildResult.log_path } else { To-RepoRelativePath -Path $buildLogPath }
        configure_log_path = if ($null -ne $ConfigureResult) { $ConfigureResult.log_path } else { To-RepoRelativePath -Path $configureLogPath }
        blocker_kind = if ($closureAvailable) { "none" } else { $BlockerKind }
        blocker_component = if ($closureAvailable) { "none" } else { "LLVM Support CrashRecoveryContext wasm32-wasip1 signal support" }
        blocker_reason = if ($closureAvailable) { "none" } else { $BlockerReason }
        first_error = $FirstError
    }
}

function Classify-Blocker {
    param(
        [string]$Text,
        [string]$DefaultKind,
        [string]$DefaultReason
    )

    $firstError = $null
    $errorMatch = [regex]::Match($Text, '(?m)^\S.*?:\d+:\d+:\s+error:\s+(.+)$')
    if ($errorMatch.Success) {
        $firstError = $errorMatch.Value.Trim()
    }

    if ($Text.Contains("CrashRecoveryContext.cpp") -and
        ($Text.Contains("sigaction") -or
            $Text.Contains("sigemptyset") -or
            $Text.Contains("sigaddset") -or
            $Text.Contains("sigprocmask"))) {
        return [ordered]@{
            kind = "target_llvm_library_closure_blocked_at_wasi_signal_sigaction"
            reason = "The wasm32-wasip1 LLVM closure build now passes the WASI machine/endian shim, LLVM_ON_UNIX FileSystem/Process configuration, and the WASI setjmp/signal header guards via -D_WASI_EMULATED_SIGNAL plus -mllvm -wasm-enable-sjlj. It then reaches LLVM Support's CrashRecoveryContext.cpp and blocks because the WASI SDK signal emulation headers do not expose the POSIX sigaction/sigemptyset/sigaddset/sigprocmask surface that LLVM Support's Unix crash-recovery path requires. No target-compatible LLVM archive closure was emitted."
            first_error = $firstError
        }
    }
    if ($Text.Contains("machine/endian.h")) {
        return [ordered]@{
            kind = "target_llvm_library_closure_blocked_at_wasi_machine_endian_header"
            reason = "The wasm32-wasip1 LLVM closure build reached LLVM Support headers but the WASI SDK did not provide machine/endian.h. The repo shim must be present before LLVM Support can compile for this target."
            first_error = $firstError
        }
    }
    if ($Text.Contains("getSize") -or $Text.Contains("EnvPathSeparator")) {
        return [ordered]@{
            kind = "target_llvm_library_closure_blocked_at_llvm_on_unix_config"
            reason = "The wasm32-wasip1 LLVM closure build reached LLVM Support with a Generic CMake system configuration that did not select LLVM's Unix support surface. LLVM_ON_UNIX must be configured for the target build."
            first_error = $firstError
        }
    }

    return [ordered]@{
        kind = $DefaultKind
        reason = $DefaultReason
        first_error = $firstError
    }
}

New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

$missing = @(
    $llvmSourceDir,
    $llvmBuildDir,
    $cmake,
    $ninja,
    $wasiClang,
    $wasiClangxx,
    $shimIncludeDir,
    $shimEndianHeader
) | Where-Object { -not (Test-Path -LiteralPath $_) }

if (@($missing).Count -gt 0) {
    $reason = "Cannot attempt wasm32-wasip1 LLVM library closure; missing required path(s): $(@($missing) -join ', ')"
    $report = New-Report `
        -PrerequisitesPresent $false `
        -ConfigureResult $null `
        -BuildResult $null `
        -TargetArchives @() `
        -BlockerKind "target_llvm_library_closure_prerequisite_missing" `
        -BlockerReason $reason `
        -FirstError $null
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
    Write-Host "target LLVM closure report: $reportPath"
    Write-Host "closure_available=false"
    Write-Host "blocker_kind=$($report.blocker_kind)"
    exit 0
}

$shimFlag = "-I$($shimIncludeDir.Replace('\', '/'))"
$commonFlags = "--target=$targetTriple -ffunction-sections -fdata-sections -fno-exceptions -DLLVM_ON_UNIX -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_MMAN -D_WASI_EMULATED_PROCESS_CLOCKS -D__wasilibc_unmodified_upstream -DHAVE_SYS_MMAN_H=1 -mllvm -wasm-enable-sjlj $shimFlag"

$configureResult = Invoke-CapturedNative -LogPath $configureLogPath -Command {
    & $cmake `
        -S $llvmSourceDir `
        -B $llvmBuildDir `
        "-DCMAKE_C_FLAGS=$commonFlags" `
        "-DCMAKE_CXX_FLAGS=$commonFlags" `
        -DLLVM_HOST_TRIPLE=$targetTriple `
        -DLLVM_DEFAULT_TARGET_TRIPLE=$targetTriple `
        -DLLVM_TARGETS_TO_BUILD=WebAssembly `
        -DLLVM_ENABLE_CRASH_OVERRIDES=OFF `
        -DLLVM_ENABLE_THREADS=OFF `
        -DLLVM_ENABLE_PROJECTS=lld `
        -DLLVM_TOOL_LLD_BUILD=ON
}

$buildResult = $null
if ($configureResult.exit_code -eq 0) {
    $buildResult = Invoke-CapturedNative -LogPath $buildLogPath -Command {
        & $ninja -C $llvmBuildDir @requiredComponents
    }
}

$targetArchives = Get-ArchiveIdentities -Names $requiredComponents
$builtArchiveComponents = @($targetArchives | ForEach-Object { $_.component } | Select-Object -Unique)
$missingRequiredComponents = @(
    $requiredComponents | Where-Object {
        $component = $_
        -not ($builtArchiveComponents -contains $component)
    }
)

if ($configureResult.exit_code -ne 0) {
    $classification = Classify-Blocker `
        -Text $configureResult.text `
        -DefaultKind "target_llvm_library_closure_configure_failed" `
        -DefaultReason "Configuring LLVM as a wasm32-wasip1 target library closure failed; inspect .rouwdi/codegen-llvm-probe/target-llvm-closure-configure.log."
} elseif ($null -eq $buildResult) {
    $classification = [ordered]@{
        kind = "target_llvm_library_closure_build_not_attempted"
        reason = "LLVM configure completed without launching the target closure build."
        first_error = $null
    }
} elseif ($buildResult.exit_code -ne 0) {
    $classification = Classify-Blocker `
        -Text $buildResult.text `
        -DefaultKind "target_llvm_library_closure_build_failed" `
        -DefaultReason "Building the wasm32-wasip1 LLVM library closure failed; inspect .rouwdi/codegen-llvm-probe/target-llvm-closure-build.log."
} elseif (@($targetArchives).Count -eq 0) {
    $classification = [ordered]@{
        kind = "target_llvm_library_closure_archive_not_emitted"
        reason = "LLVM's wasm32-wasip1 closure build exited 0 for the required closure components, but no target-compatible archive was found in the configured LLVM build tree."
        first_error = $null
    }
} elseif (@($missingRequiredComponents).Count -gt 0) {
    $classification = [ordered]@{
        kind = "target_llvm_library_closure_incomplete"
        reason = "LLVM's wasm32-wasip1 closure build exited 0 but did not emit all required target-compatible archives. Missing components: $(@($missingRequiredComponents) -join ', ')."
        first_error = $null
    }
} else {
    $classification = [ordered]@{
        kind = "none"
        reason = "none"
        first_error = $null
    }
}

$report = New-Report `
    -PrerequisitesPresent $true `
    -ConfigureResult $configureResult `
    -BuildResult $buildResult `
    -TargetArchives @($targetArchives) `
    -BlockerKind $classification.kind `
    -BlockerReason $classification.reason `
    -FirstError $classification.first_error
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8

Write-Host "target LLVM closure report: $reportPath"
Write-Host "llvm_wrapper_target=$targetTriple"
Write-Host "artifact_kind=staticlib-closure"
Write-Host "closure_available=$($report.closure_available)"
Write-Host "blocker_kind=$($report.blocker_kind)"
Write-Host "log_path=$($report.log_path)"
exit 0
