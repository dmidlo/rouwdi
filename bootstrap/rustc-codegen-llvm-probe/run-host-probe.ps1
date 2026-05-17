$ErrorActionPreference = "Stop"

$repo = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$stage0Cargo = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage0\bin\cargo.exe"
$stage1Rustc = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1\bin\rustc.exe"
$manifest = Join-Path $repo "bootstrap\rustc-codegen-llvm-probe\Cargo.toml"
$targetDir = Join-Path $repo ".rouwdi\codegen-llvm-probe\host-target"
$reportDir = Join-Path $repo ".rouwdi\codegen-llvm-probe"
$reportPath = Join-Path $reportDir "host-probe-report.json"
$logPath = Join-Path $reportDir "host-probe.log"
$llvmConfig = Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\ci-llvm\bin\llvm-config.exe"
$wrapperCandidates = @(
    (Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1-rustc\x86_64-pc-windows-msvc\release\build\rustc_llvm-0d4f61ce596f94b4\out"),
    (Join-Path $repo "third_party\rust\build\x86_64-pc-windows-msvc\stage1-rustc\x86_64-pc-windows-msvc\release\build\rustc_llvm-df1e4c80cc0afe5c\out")
)

function Split-ToolArgs {
    param([AllowEmptyString()][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return @()
    }

    return @($Value.Trim() -split '\s+' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function Invoke-LlvmConfig {
    param([string[]]$Arguments)

    $output = & $llvmConfig @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "llvm-config $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
    return ($output -join " ").Trim()
}

function Resolve-LibraryPaths {
    param(
        [string]$LibDir,
        [string[]]$Libraries
    )

    $resolved = New-Object System.Collections.Generic.List[string]
    foreach ($library in $Libraries) {
        if ([System.IO.Path]::IsPathRooted($library)) {
            $resolved.Add($library)
            continue
        }

        $candidate = Join-Path $LibDir $library
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            $resolved.Add($candidate)
        } else {
            $resolved.Add($library)
        }
    }
    return @($resolved)
}

function Get-UnresolvedSymbols {
    param([string[]]$OutputLines)

    $symbols = New-Object System.Collections.Generic.List[string]
    $knownPatterns = @(
        "LLVMBuildSelect",
        "LLVMBuildRet",
        "llvm::Linker::Linker",
        "llvm::MemoryBuffer::getMemBufferCopy",
        "llvm::LLVMContext::LLVMContext",
        "llvm::OptimizationLevel::O2",
        "llvm::TargetLibraryAnalysis::Key"
    )

    foreach ($line in $OutputLines) {
        if ($line -notmatch "unresolved external symbol") {
            continue
        }

        foreach ($pattern in $knownPatterns) {
            if ($line.Contains($pattern) -and -not $symbols.Contains($pattern)) {
                $symbols.Add($pattern)
            }
        }

        if ($line -match 'unresolved external symbol\s+"([^"]+)"') {
            $symbol = $Matches[1].Trim()
            if (-not [string]::IsNullOrWhiteSpace($symbol) -and -not $symbols.Contains($symbol)) {
                $symbols.Add($symbol)
            }
        } elseif ($line -match 'unresolved external symbol\s+([^\s]+)') {
            $symbol = $Matches[1].Trim()
            if (-not [string]::IsNullOrWhiteSpace($symbol) -and -not $symbols.Contains($symbol)) {
                $symbols.Add($symbol)
            }
        }
    }

    return @($symbols)
}

if (!(Test-Path $stage0Cargo)) {
    throw "stage0 cargo not found at $stage0Cargo; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $stage1Rustc)) {
    throw "stage1 rustc not found at $stage1Rustc; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
if (!(Test-Path $llvmConfig)) {
    throw "llvm-config not found at $llvmConfig; run python x.py check compiler/rustc_codegen_llvm --stage 1 -v first"
}
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
$wrapperDir = $wrapperCandidates | Where-Object { Test-Path (Join-Path $_ "llvm-wrapper.lib") } | Select-Object -First 1
if (!$wrapperDir) {
    throw "stage1 llvm-wrapper.lib not found under stage1-rustc build output"
}
$llvmLibDir = Invoke-LlvmConfig @("--libdir")
if (!(Test-Path $llvmLibDir)) {
    throw "LLVM library directory from llvm-config not found: $llvmLibDir"
}
$llvmLibs = Split-ToolArgs (Invoke-LlvmConfig @("--libs"))
$llvmLibNames = Split-ToolArgs (Invoke-LlvmConfig @("--libnames"))
$llvmSystemLibs = Split-ToolArgs (Invoke-LlvmConfig @("--system-libs"))
$llvmLdFlags = Split-ToolArgs (Invoke-LlvmConfig @("--ldflags"))
$wrapperPath = Join-Path $wrapperDir "llvm-wrapper.lib"
$resolvedLlvmLibraries = Resolve-LibraryPaths -LibDir $llvmLibDir -Libraries $llvmLibNames
$resolvedLibraries = @($wrapperPath) + $resolvedLlvmLibraries + $llvmSystemLibs
$linkSearchPaths = @($wrapperDir, $llvmLibDir)
$linkArgs = @("/LIBPATH:$wrapperDir", "/LIBPATH:$llvmLibDir", $wrapperPath) + $llvmLdFlags + $llvmLibs + $llvmSystemLibs

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
Write-Host "stage1 llvm-wrapper.lib: $wrapperPath"
Write-Host "llvm-config path: $llvmConfig"
Write-Host "CI LLVM lib directory: $llvmLibDir"
Write-Host "LLVM libraries from llvm-config: $($llvmLibNames.Count)"
Write-Host "MSVC system libraries from llvm-config: $($llvmSystemLibs -join ' ')"
$env:RUSTFLAGS = (@("-L native=$wrapperDir", "-L native=$llvmLibDir") + ($linkArgs | ForEach-Object { "-C link-arg=$_" })) -join " "

$probeArgs = @(
    "--json",
    "--compile-unit-id", "app:rust:app:wasm32-wasip1",
    "--crate-identity", "rouwdi_payload",
    "--target-triple", "wasm32-wasip1",
    "--target-spec", "rustc_target::spec::wasm32_wasip1",
    "--mir-body-hash", "a5e137ef6793c0b8",
    "--mono-item-count", "1",
    "--mono-item-graph-hash", "bec5817d61819666",
    "--mono-item", "fn:rouwdi_payload::main"
)

$previousErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
try {
    $probeOutput = & $stage0Cargo run --manifest-path $manifest -- @probeArgs 2>&1
    $probeExitCode = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $previousErrorActionPreference
}
$probeOutputLines = @($probeOutput | ForEach-Object { $_.ToString() })
$probeOutputLines | Set-Content -LiteralPath $logPath -Encoding utf8
$remainingUnresolvedSymbols = Get-UnresolvedSymbols -OutputLines $probeOutputLines
$backendConstructorResult = $false
$backendName = $null
$codegenContactState = $null
$llvmModuleSetup = $null
$targetMachineSetup = $null
$monoProofConsumed = $false
if ($probeExitCode -eq 0) {
    $jsonStart = -1
    $jsonEnd = -1
    for ($i = 0; $i -lt $probeOutputLines.Count; $i++) {
        if ($probeOutputLines[$i].TrimStart().StartsWith("{")) {
            $jsonStart = $i
            break
        }
    }
    for ($i = $probeOutputLines.Count - 1; $i -ge 0; $i--) {
        if ($probeOutputLines[$i].TrimEnd().EndsWith("}")) {
            $jsonEnd = $i
            break
        }
    }
    $probeJsonText = if ($jsonStart -ge 0 -and $jsonEnd -ge $jsonStart) {
        ($probeOutputLines[$jsonStart..$jsonEnd] -join "`n")
    } else {
        ($probeOutputLines -join "`n")
    }
    try {
        $probeJson = $probeJsonText | ConvertFrom-Json
        $backendConstructorResult = ($probeJson.backend_constructed -eq $true)
        $backendName = $probeJson.backend_name
        $codegenContactState = $probeJson.codegen_contact_state
        $llvmModuleSetup = $probeJson.llvm_module_setup
        $targetMachineSetup = $probeJson.target_machine_setup
        $monoProofConsumed = ($probeJson.mono_proof_consumed -eq $true)
    } catch {
        $backendConstructorResult = $false
    }
}

$state = if ($probeExitCode -eq 0 -and $backendConstructorResult) {
    "host_codegen_probe_backend_constructed"
} elseif ($remainingUnresolvedSymbols.Count -gt 0) {
    $firstSymbol = ($remainingUnresolvedSymbols[0] -replace '[^A-Za-z0-9_:]+', '_').Trim("_")
    "host_codegen_probe_blocked_at_unresolved_llvm_symbol_$firstSymbol"
} else {
    "llvm_library_closure_resolved_probe_failed_before_backend_constructor"
}

$report = [ordered]@{
    schema_version = 1
    probe_name = "rustc_codegen_llvm_host_backend_probe"
    state = $state
    llvm_config_path = $llvmConfig
    llvm_libdir = $llvmLibDir
    llvm_libs = @($llvmLibs)
    llvm_libnames = @($llvmLibNames)
    llvm_system_libs = @($llvmSystemLibs)
    llvm_ldflags = @($llvmLdFlags)
    llvm_wrapper_path = $wrapperPath
    link_search_paths = @($linkSearchPaths)
    resolved_libraries = @($resolvedLibraries)
    remaining_unresolved_symbols = @($remainingUnresolvedSymbols)
    host_probe_exit_code = $probeExitCode
    backend_constructor_result = $backendConstructorResult
    backend_name = $backendName
    codegen_contact_state = $codegenContactState
    mono_proof_consumed = $monoProofConsumed
    llvm_module_setup = $llvmModuleSetup
    target_machine_setup = $targetMachineSetup
    log_path = ".rouwdi/codegen-llvm-probe/host-probe.log"
}
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $reportPath -Encoding utf8

Write-Host "host probe report: $reportPath"
$probeOutputLines | ForEach-Object { Write-Host $_ }
exit $probeExitCode
