$ErrorActionPreference = "Stop"

$repo = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$reportDir = Join-Path $repo ".rouwdi\codegen-llvm-probe"
$workDir = Join-Path $reportDir "target-llvm-wrapper"
$objectDir = Join-Path $workDir "obj"
$archiveDir = Join-Path $workDir "lib"
$reportPath = Join-Path $reportDir "target-llvm-wrapper-report.json"
$wrapperSourceDir = Join-Path $repo "third_party\rust\compiler\rustc_llvm\llvm-wrapper"
$llvmConfig = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\ci-llvm\bin\llvm-config.exe"
$wasiSdk = Join-Path $repo ".rouwdi\tools\wasi-sdk\wasi-sdk-33.0-x86_64-windows"
$wasiClangxx = Join-Path $wasiSdk "bin\wasm32-wasip1-clang++.exe"
$wasiAr = Join-Path $wasiSdk "bin\llvm-ar.exe"
$wasiRanlib = Join-Path $wasiSdk "bin\llvm-ranlib.exe"
$wasiSysroot = Join-Path $wasiSdk "share\wasi-sysroot"
$archivePath = Join-Path $archiveDir "libllvm-wrapper.a"
$targetTriple = "wasm32-wasip1"

function To-RepoRelativePath {
    param([string]$Path)

    $resolved = if (Test-Path -LiteralPath $Path) {
        (Resolve-Path -LiteralPath $Path).Path
    } else {
        [System.IO.Path]::GetFullPath($Path)
    }
    $root = $repo.Path.TrimEnd([char[]]"\/")
    $rootPrefix = $root + [System.IO.Path]::DirectorySeparatorChar
    if ($resolved.Equals($root, [System.StringComparison]::OrdinalIgnoreCase)) {
        return "."
    }
    if ($resolved.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $resolved.Substring($rootPrefix.Length).Replace("\", "/")
    }
    return $resolved.Replace("\", "/")
}

function Get-ArtifactIdentity {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $null
    }

    $item = Get-Item -LiteralPath $Path
    $hash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    return [ordered]@{
        path = To-RepoRelativePath -Path $Path
        sha256 = $hash
        size_bytes = [int64]$item.Length
    }
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

function Invoke-LlvmConfig {
    param([string[]]$Arguments)

    $output = & $llvmConfig @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "llvm-config $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
    return ($output -join " ").Trim()
}

function Split-ToolArgs {
    param([AllowEmptyString()][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return @()
    }

    return @($Value.Trim() -split '\s+' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function New-Manifest {
    param(
        [bool]$Attempted,
        [object]$CompileAttempts,
        [object]$ArchiveIdentity,
        [object]$ArchiveResult,
        [string]$BlockerKind,
        [string]$BlockerReason,
        [object]$LlvmNativeLibraries,
        [object]$LlvmComponents
    )

    $archiveExists = $null -ne $ArchiveIdentity
    $targetLlvmLibraryClosureAvailable = $false
    $targetLoadable = $archiveExists -and $targetLlvmLibraryClosureAvailable

    return [ordered]@{
        schema_version = 1
        probe_name = "rustc_llvm_target_wrapper_build"
        llvm_wrapper_target = $targetTriple
        artifact_kind = "staticlib"
        path = if ($archiveExists) { $ArchiveIdentity.path } else { (To-RepoRelativePath -Path $archivePath) }
        sha256 = if ($archiveExists) { $ArchiveIdentity.sha256 } else { $null }
        size_bytes = if ($archiveExists) { $ArchiveIdentity.size_bytes } else { $null }
        built_by = "bootstrap/rustc-codegen-llvm-probe/build-target-llvm-wrapper.ps1"
        build_attempted = $Attempted
        build_command = "wasm32-wasip1-clang++ -c rustc_llvm/llvm-wrapper/*.cpp; llvm-ar crs libllvm-wrapper.a"
        wrapper_sources = @(
            "PassWrapper.cpp",
            "RustWrapper.cpp",
            "CoverageMappingWrapper.cpp",
            "SymbolWrapper.cpp",
            "Linker.cpp"
        )
        compile_defines = @(
            "LLVM_ON_UNIX",
            "NDEBUG",
            "__STDC_FORMAT_MACROS",
            "__STDC_LIMIT_MACROS",
            "__STDC_CONSTANT_MACROS"
        )
        compile_attempts = @($CompileAttempts)
        archive_exit_code = if ($null -ne $ArchiveResult) { $ArchiveResult.exit_code } else { $null }
        linked_into = if ($targetLoadable) { "bootstrap/rustc-codegen-llvm-probe wasm32 backend payload" } else { "not_linked" }
        target_loadable = $targetLoadable
        wrapper_archive_emitted = $archiveExists
        target_object_archive = $archiveExists
        target_llvm_library_closure_available = $targetLlvmLibraryClosureAvailable
        target_llvm_library_closure = [ordered]@{
            required = $true
            status = "missing_target_compatible_llvm_libraries"
            target_triple = $targetTriple
            host_ci_llvm_reused_as_target = $false
            native_ci_llvm_libraries = @($LlvmNativeLibraries)
            components = @($LlvmComponents)
            reason = "rustc_llvm's wrapper archive can be compiled as wasm32-wasip1 object code, but the executable backend payload still needs a wasm32-wasip1 LLVM library closure. The repo currently only has native MSVC CI LLVM libraries, which are host evidence and are not target-loadable into the Wasm backend payload."
        }
        blocker_kind = $BlockerKind
        blocker_reason = $BlockerReason
    }
}

New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
New-Item -ItemType Directory -Force -Path $objectDir | Out-Null
New-Item -ItemType Directory -Force -Path $archiveDir | Out-Null

foreach ($requiredPath in @($wrapperSourceDir, $llvmConfig, $wasiClangxx, $wasiAr, $wasiRanlib, $wasiSysroot)) {
    if (-not (Test-Path -LiteralPath $requiredPath)) {
        $manifest = New-Manifest `
            -Attempted $false `
            -CompileAttempts @() `
            -ArchiveIdentity $null `
            -ArchiveResult $null `
            -BlockerKind "target_llvm_wrapper_prerequisite_missing" `
            -BlockerReason "Required path is missing: $requiredPath" `
            -LlvmNativeLibraries @() `
            -LlvmComponents @()
        $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
        Write-Host "target llvm-wrapper report: $reportPath"
        exit 0
    }
}

$llvmIncludeDir = Invoke-LlvmConfig @("--includedir")
$llvmLibDir = Invoke-LlvmConfig @("--libdir")
$llvmLibNames = Split-ToolArgs (Invoke-LlvmConfig @("--libnames"))
$llvmComponents = Split-ToolArgs (Invoke-LlvmConfig @("--components"))
$supportedComponents = @(
    "ipo",
    "bitreader",
    "bitwriter",
    "linker",
    "asmparser",
    "lto",
    "coverage",
    "instrumentation",
    "x86",
    "arm",
    "aarch64",
    "amdgpu",
    "avr",
    "loongarch",
    "m68k",
    "csky",
    "mips",
    "powerpc",
    "systemz",
    "webassembly",
    "msp430",
    "sparc",
    "nvptx",
    "hexagon",
    "riscv",
    "xtensa",
    "bpf"
)
$componentDefines = @(
    $llvmComponents |
        Where-Object { $supportedComponents -contains $_ } |
        ForEach-Object { "-DLLVM_COMPONENT_$($_.ToUpperInvariant())" }
)
$nativeLlvmLibraries = @(
    $llvmLibNames | ForEach-Object {
        $candidate = Join-Path $llvmLibDir $_
        [ordered]@{
            name = $_
            path = if (Test-Path -LiteralPath $candidate -PathType Leaf) { To-RepoRelativePath -Path $candidate } else { $_ }
            target_compatible = $false
            reason = "native CI LLVM library; host evidence only"
        }
    }
)

$sources = @(
    "PassWrapper.cpp",
    "RustWrapper.cpp",
    "CoverageMappingWrapper.cpp",
    "SymbolWrapper.cpp",
    "Linker.cpp"
)
$objects = New-Object System.Collections.Generic.List[string]
$compileAttempts = New-Object System.Collections.Generic.List[object]
$compileFailed = $false

foreach ($source in $sources) {
    $sourcePath = Join-Path $wrapperSourceDir $source
    $objectPath = Join-Path $objectDir ($source -replace '\.cpp$', '.o')
    $logPath = Join-Path $workDir "$($source -replace '\.cpp$', '').compile.log"
    $result = Invoke-CapturedNative -LogPath $logPath -Command {
        & $wasiClangxx `
            --target=$targetTriple `
            --sysroot=$wasiSysroot `
            -std=c++17 `
            -fno-exceptions `
            -fno-rtti `
            -D__STDC_FORMAT_MACROS `
            -D__STDC_LIMIT_MACROS `
            -D__STDC_CONSTANT_MACROS `
            -DNDEBUG `
            -DLLVM_ON_UNIX `
            @componentDefines `
            "-I$wrapperSourceDir" `
            "-I$llvmIncludeDir" `
            -c $sourcePath `
            -o $objectPath
    }
    $identity = Get-ArtifactIdentity -Path $objectPath
    $compileAttempts.Add([ordered]@{
        source = "third_party/rust/compiler/rustc_llvm/llvm-wrapper/$source"
        object_path = if ($null -ne $identity) { $identity.path } else { $null }
        object_sha256 = if ($null -ne $identity) { $identity.sha256 } else { $null }
        object_size_bytes = if ($null -ne $identity) { $identity.size_bytes } else { $null }
        exit_code = $result.exit_code
        log_path = $result.log_path
        target_triple = $targetTriple
    })
    if ($result.exit_code -ne 0 -or $null -eq $identity) {
        $compileFailed = $true
        break
    }
    $objects.Add($objectPath)
}

$archiveResult = $null
if (-not $compileFailed) {
    Remove-Item -LiteralPath $archivePath -Force -ErrorAction SilentlyContinue
    $archiveLogPath = Join-Path $workDir "archive.log"
    $archiveResult = Invoke-CapturedNative -LogPath $archiveLogPath -Command {
        & $wasiAr crs $archivePath @($objects.ToArray())
        if ($LASTEXITCODE -eq 0) {
            & $wasiRanlib $archivePath
        }
    }
}

$archiveIdentity = Get-ArtifactIdentity -Path $archivePath
$blockerKind = if ($compileFailed) {
    "wasm32_llvm_wrapper_compile_failed"
} elseif ($null -eq $archiveIdentity) {
    "wasm32_llvm_wrapper_archive_not_emitted"
} else {
    "wasm_codegen_payload_blocked_at_target_llvm_library_closure"
}
$blockerReason = if ($compileFailed) {
    "Compiling Rust's rustc_llvm wrapper sources with WASI clang++ failed; inspect the per-source compile logs under .rouwdi/codegen-llvm-probe/target-llvm-wrapper."
} elseif ($null -eq $archiveIdentity) {
    "The wrapper C++ sources compiled, but llvm-ar did not emit libllvm-wrapper.a."
} else {
    "A wasm32-wasip1 llvm-wrapper archive was emitted, but it is not sufficient for executable rustc_codegen_llvm payload linkage because the repository does not yet contain a wasm32-wasip1 target-compatible LLVM library closure. Native MSVC CI LLVM libraries are host evidence only and are not marked target-loadable."
}

$manifest = New-Manifest `
    -Attempted $true `
    -CompileAttempts @($compileAttempts.ToArray()) `
    -ArchiveIdentity $archiveIdentity `
    -ArchiveResult $archiveResult `
    -BlockerKind $blockerKind `
    -BlockerReason $blockerReason `
    -LlvmNativeLibraries @($nativeLlvmLibraries) `
    -LlvmComponents @($llvmComponents)
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8

Write-Host "target llvm-wrapper report: $reportPath"
Write-Host "llvm_wrapper_target=$targetTriple"
Write-Host "artifact_kind=staticlib"
Write-Host "path=$($manifest.path)"
Write-Host "sha256=$($manifest.sha256)"
Write-Host "size_bytes=$($manifest.size_bytes)"
Write-Host "target_loadable=$($manifest.target_loadable)"
Write-Host "blocker_kind=$($manifest.blocker_kind)"
exit 0
