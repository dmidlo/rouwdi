param(
    [string]$RepoRoot = "",
    [switch]$NoDownload
)

$ErrorActionPreference = "Stop"

$NinjaVersion = "1.13.2"
$NinjaAssetName = "ninja-win.zip"
$NinjaArchiveName = "ninja-win-$NinjaVersion.zip"
$NinjaAssetUrl = "https://github.com/ninja-build/ninja/releases/download/v$NinjaVersion/$NinjaAssetName"
$NinjaAssetSha256 = "07fc8261b42b20e71d1720b39068c2e14ffcee6396b76fb7a795fb460b78dc65"
$CMakeVersion = "4.3.2"
$CMakeAssetName = "cmake-$CMakeVersion-windows-x86_64.zip"
$CMakeAssetUrl = "https://github.com/Kitware/CMake/releases/download/v$CMakeVersion/$CMakeAssetName"
$CMakeAssetSha256 = "83d20c23f5c5f64b3b328785e35b23c532e33057a97ed6294acaca3781b78a01"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
} else {
    $RepoRoot = (Resolve-Path $RepoRoot).Path
}

function Join-RepoPath([string]$Path) {
    return (Join-Path $RepoRoot $Path)
}

function ConvertTo-TomlPath([string]$Path) {
    return $Path.Replace("\", "\\")
}

function ConvertTo-RelativeRepoPath([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path)) {
        return ""
    }
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    if ($resolved.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $resolved.Substring($RepoRoot.Length + 1)
    }
    return $resolved
}

function Write-Utf8NoBom([string]$Path, [string]$Content) {
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $encoding)
}

function Get-ToolRecord([string]$Name, [string]$ExeName) {
    $command = Get-Command $ExeName -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        return [pscustomobject]@{
            name = $Name
            found = $false
            path = ""
            version = ""
        }
    }

    $path = $command.Source
    if ([string]::IsNullOrWhiteSpace($path)) {
        $path = $command.Definition
    }

    $version = ""
    try {
        $version = (& $path --version 2>&1 | Select-Object -First 1)
    } catch {
        $version = $_.Exception.Message
    }

    return [pscustomobject]@{
        name = $Name
        found = $true
        path = $path
        version = [string]$version
    }
}

function Test-NinjaRoot([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path) -or !(Test-Path -LiteralPath $Path -PathType Container)) {
        return $false
    }
    $exe = Join-Path $Path "ninja.exe"
    if (!(Test-Path -LiteralPath $exe -PathType Leaf)) {
        return $false
    }
    $version = (& $exe --version 2>&1 | Select-Object -First 1)
    return [string]$version -eq $NinjaVersion
}

function Ensure-Ninja {
    $toolsRoot = Join-RepoPath ".rouwdi\tools\ninja\ninja-$NinjaVersion-win"
    $archivePath = Join-RepoPath ".rouwdi\tool-cache\ninja\$NinjaArchiveName"

    if (Test-NinjaRoot $toolsRoot) {
        return [pscustomobject]@{
            status = "ready"
            source = "repo_local_existing"
            root = (Resolve-Path -LiteralPath $toolsRoot).Path
            exe = (Resolve-Path -LiteralPath (Join-Path $toolsRoot "ninja.exe")).Path
            archive = $archivePath
            archive_sha256 = $NinjaAssetSha256
            version = $NinjaVersion
        }
    }

    if ($NoDownload) {
        throw "Repo-local Ninja $NinjaVersion was not found and -NoDownload was supplied."
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $archivePath) | Out-Null
    New-Item -ItemType Directory -Force -Path $toolsRoot | Out-Null

    if (!(Test-Path -LiteralPath $archivePath -PathType Leaf)) {
        & curl.exe -L --fail --retry 3 --output $archivePath $NinjaAssetUrl
        if ($LASTEXITCODE -ne 0) {
            throw "curl failed while downloading $NinjaAssetUrl with exit code $LASTEXITCODE"
        }
    }

    $actualSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $archivePath).Hash.ToLowerInvariant()
    if ($actualSha -ne $NinjaAssetSha256) {
        throw "Ninja archive hash mismatch: expected $NinjaAssetSha256, got $actualSha"
    }

    Expand-Archive -LiteralPath $archivePath -DestinationPath $toolsRoot -Force
    if (!(Test-NinjaRoot $toolsRoot)) {
        throw "Downloaded Ninja archive did not install ninja.exe $NinjaVersion under $toolsRoot"
    }

    return [pscustomobject]@{
        status = "ready"
        source = "downloaded_github_release"
        root = (Resolve-Path -LiteralPath $toolsRoot).Path
        exe = (Resolve-Path -LiteralPath (Join-Path $toolsRoot "ninja.exe")).Path
        archive = $archivePath
        archive_sha256 = $actualSha
        version = $NinjaVersion
    }
}

function Test-CMakeRoot([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path) -or !(Test-Path -LiteralPath $Path -PathType Container)) {
        return $false
    }
    $exe = Join-Path $Path "bin\cmake.exe"
    if (!(Test-Path -LiteralPath $exe -PathType Leaf)) {
        return $false
    }
    $version = (& $exe --version 2>&1 | Select-Object -First 1)
    return [string]$version -eq "cmake version $CMakeVersion"
}

function Ensure-CMake {
    $toolsRoot = Join-RepoPath ".rouwdi\tools\cmake\cmake-$CMakeVersion-windows-x86_64"
    $archivePath = Join-RepoPath ".rouwdi\tool-cache\cmake\$CMakeAssetName"

    if (Test-CMakeRoot $toolsRoot) {
        return [pscustomobject]@{
            status = "ready"
            source = "repo_local_existing"
            root = (Resolve-Path -LiteralPath $toolsRoot).Path
            exe = (Resolve-Path -LiteralPath (Join-Path $toolsRoot "bin\cmake.exe")).Path
            archive = $archivePath
            archive_sha256 = $CMakeAssetSha256
            version = $CMakeVersion
        }
    }

    if ($NoDownload) {
        throw "Repo-local CMake $CMakeVersion was not found and -NoDownload was supplied."
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $archivePath) | Out-Null
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $toolsRoot) | Out-Null

    if (!(Test-Path -LiteralPath $archivePath -PathType Leaf)) {
        & curl.exe -L --fail --retry 3 --output $archivePath $CMakeAssetUrl
        if ($LASTEXITCODE -ne 0) {
            throw "curl failed while downloading $CMakeAssetUrl with exit code $LASTEXITCODE"
        }
    }

    $actualSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $archivePath).Hash.ToLowerInvariant()
    if ($actualSha -ne $CMakeAssetSha256) {
        throw "CMake archive hash mismatch: expected $CMakeAssetSha256, got $actualSha"
    }

    Expand-Archive -LiteralPath $archivePath -DestinationPath (Split-Path -Parent $toolsRoot) -Force
    if (!(Test-CMakeRoot $toolsRoot)) {
        throw "Downloaded CMake archive did not install cmake.exe $CMakeVersion under $toolsRoot"
    }

    return [pscustomobject]@{
        status = "ready"
        source = "downloaded_github_release"
        root = (Resolve-Path -LiteralPath $toolsRoot).Path
        exe = (Resolve-Path -LiteralPath (Join-Path $toolsRoot "bin\cmake.exe")).Path
        archive = $archivePath
        archive_sha256 = $actualSha
        version = $CMakeVersion
    }
}

function Find-WasiSdkRoot {
    $candidates = New-Object System.Collections.Generic.List[string]
    if (![string]::IsNullOrWhiteSpace($env:WASI_SDK_PATH)) {
        $candidates.Add($env:WASI_SDK_PATH)
    }
    $candidates.Add((Join-RepoPath ".rouwdi\tools\wasi-sdk\wasi-sdk-33.0-x86_64-windows"))

    foreach ($candidate in $candidates) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }
        $clang = Join-Path $candidate "bin\wasm32-wasip1-clang.exe"
        $sysroot = Join-Path $candidate "share\wasi-sysroot"
        if ((Test-Path -LiteralPath $clang -PathType Leaf) -and (Test-Path -LiteralPath $sysroot -PathType Container)) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    return $null
}

function Upsert-TomlSetting([string[]]$Lines, [string]$SectionHeader, [string]$Key, [string]$Setting) {
    $sectionStart = -1
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        if ($Lines[$i].Trim() -eq $SectionHeader) {
            $sectionStart = $i
            break
        }
    }

    if ($sectionStart -lt 0) {
        return @($Lines + "" + $SectionHeader + $Setting)
    }

    $sectionEnd = $Lines.Count
    for ($i = $sectionStart + 1; $i -lt $Lines.Count; $i++) {
        if ($Lines[$i] -match "^\s*\[.+\]\s*$") {
            $sectionEnd = $i
            break
        }
    }

    $settingIndex = -1
    for ($i = $sectionStart + 1; $i -lt $sectionEnd; $i++) {
        if ($Lines[$i] -match "^\s*$([regex]::Escape($Key))\s*=") {
            $settingIndex = $i
            break
        }
    }

    if ($settingIndex -ge 0) {
        $Lines[$settingIndex] = $Setting
        return $Lines
    }

    $before = @($Lines[0..$sectionStart])
    $after = @()
    if ($sectionStart + 1 -lt $Lines.Count) {
        $after = @($Lines[($sectionStart + 1)..($Lines.Count - 1)])
    }
    return @($before + $Setting + $after)
}

function Set-Stage2BootstrapConfig([string]$WasiSdkRoot, [string]$WasiRoot) {
    $bootstrapPath = Join-RepoPath "third_party\rust\bootstrap.toml"
    $lines = @()
    if (Test-Path -LiteralPath $bootstrapPath -PathType Leaf) {
        $lines = @(Get-Content -LiteralPath $bootstrapPath)
    } else {
        $lines = @(
            "# Generated by bootstrap/provision-stage2-wasm-host-tooling.ps1.",
            "# This file is ignored by third_party/rust/.gitignore."
        )
    }

    $lines = Upsert-TomlSetting $lines "[llvm]" "download-ci-llvm" 'download-ci-llvm = true'
    $lines = Upsert-TomlSetting $lines "[llvm]" "ninja" 'ninja = true'
    $lines = Upsert-TomlSetting $lines "[target.wasm32-wasip1]" "wasi-root" "wasi-root = `"$(ConvertTo-TomlPath $WasiRoot)`""
    $lines = Upsert-TomlSetting $lines "[target.wasm32-wasip1]" "cc" "cc = `"$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang.exe"))`""
    $lines = Upsert-TomlSetting $lines "[target.wasm32-wasip1]" "cxx" "cxx = `"$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang++.exe"))`""
    $lines = Upsert-TomlSetting $lines "[target.wasm32-wasip1]" "ar" "ar = `"$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\llvm-ar.exe"))`""
    $lines = Upsert-TomlSetting $lines "[target.wasm32-wasip1]" "ranlib" "ranlib = `"$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\llvm-ranlib.exe"))`""
    $lines = Upsert-TomlSetting $lines "[target.wasm32-wasip1]" "linker" "linker = `"$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang.exe"))`""

    Write-Utf8NoBom $bootstrapPath (($lines -join [Environment]::NewLine) + [Environment]::NewLine)
    return $bootstrapPath
}

function Write-ToolingToml($Ninja, $CMake, [string]$WasiSdkRoot, [string]$BootstrapPath) {
    $toolingPath = Join-RepoPath "bootstrap\tooling.toml"
    $cmakeDetected = Get-ToolRecord "cmake" "cmake"
    $clang = Get-ToolRecord "clang" "clang"
    $clangCl = Get-ToolRecord "clang-cl" "clang-cl"
    $wasmLd = Get-ToolRecord "wasm-ld" "wasm-ld"
    $lldLink = Get-ToolRecord "lld-link" "lld-link"
    $python = Get-ToolRecord "python" "python"
    $curl = Get-ToolRecord "curl" "curl.exe"
    $tar = Get-ToolRecord "tar" "tar"

    $wasiRoot = if ([string]::IsNullOrWhiteSpace($WasiSdkRoot)) { "" } else { Join-Path $WasiSdkRoot "share\wasi-sysroot" }
    $wasiClang = if ([string]::IsNullOrWhiteSpace($WasiSdkRoot)) { "" } else { Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang.exe" }

    $content = @"
schema_version = 1
milestone = "stage2_wasm_host_rustc_private_payload_route"
recorded_by = "bootstrap/provision-stage2-wasm-host-tooling.ps1"

[stage2_wasm_host_route]
command = "python x.py build compiler/rustc_span --stage 2 --host wasm32-wasip1 -v"
workdir = "third_party/rust"
bootstrap_config = "$(ConvertTo-TomlPath (ConvertTo-RelativeRepoPath $BootstrapPath))"
bootstrap_config_download_ci_llvm = true
bootstrap_config_llvm_ninja = true
bootstrap_config_wasm32_wasip1_wasi_root = "$(ConvertTo-TomlPath $wasiRoot)"
bootstrap_config_wasm32_wasip1_cc = "$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang.exe"))"
bootstrap_config_wasm32_wasip1_cxx = "$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang++.exe"))"
bootstrap_config_wasm32_wasip1_ar = "$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\llvm-ar.exe"))"
bootstrap_config_wasm32_wasip1_ranlib = "$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\llvm-ranlib.exe"))"
bootstrap_config_wasm32_wasip1_linker = "$(ConvertTo-TomlPath (Join-Path $WasiSdkRoot "bin\wasm32-wasip1-clang.exe"))"
path_prefix = "$(ConvertTo-TomlPath (ConvertTo-RelativeRepoPath $Ninja.root))"
decision = "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"

[tools.ninja]
required = true
status = "$($Ninja.status)"
source = "$($Ninja.source)"
version = "$($Ninja.version)"
path = "$(ConvertTo-TomlPath (ConvertTo-RelativeRepoPath $Ninja.exe))"
archive = "$(ConvertTo-TomlPath (ConvertTo-RelativeRepoPath $Ninja.archive))"
archive_url = "$NinjaAssetUrl"
archive_sha256 = "$($Ninja.archive_sha256)"

[tools.cmake]
required = true
required_because = "The stage2 wasm-host route entered LLVM-for-wasm32-wasip1 CMake configuration after Ninja was provisioned."
status = "$($CMake.status)"
source = "$($CMake.source)"
version = "$($CMake.version)"
path = "$(ConvertTo-TomlPath (ConvertTo-RelativeRepoPath $CMake.exe))"
archive = "$(ConvertTo-TomlPath (ConvertTo-RelativeRepoPath $CMake.archive))"
archive_url = "$CMakeAssetUrl"
archive_sha256 = "$($CMake.archive_sha256)"
detected_on_path = $($cmakeDetected.found.ToString().ToLowerInvariant())
detected_path = "$(ConvertTo-TomlPath $cmakeDetected.path)"
detected_version = "$($cmakeDetected.version)"

[tools.llvm]
download_ci_llvm_configured = true
llvm_tools_found = $($wasmLd.found.ToString().ToLowerInvariant())
wasm_ld_found = $($wasmLd.found.ToString().ToLowerInvariant())
wasm_ld_path = "$(ConvertTo-TomlPath $wasmLd.path)"
wasm_ld_version = "$($wasmLd.version)"
lld_link_found = $($lldLink.found.ToString().ToLowerInvariant())
lld_link_path = "$(ConvertTo-TomlPath $lldLink.path)"
lld_link_version = "$($lldLink.version)"

[tools.clang]
clang_found = $($clang.found.ToString().ToLowerInvariant())
clang_path = "$(ConvertTo-TomlPath $clang.path)"
clang_version = "$($clang.version)"
clang_cl_found = $($clangCl.found.ToString().ToLowerInvariant())
clang_cl_path = "$(ConvertTo-TomlPath $clangCl.path)"
clang_cl_version = "$($clangCl.version)"

[tools.python]
required = true
found = $($python.found.ToString().ToLowerInvariant())
path = "$(ConvertTo-TomlPath $python.path)"
version = "$($python.version)"

[tools.fetch_extract]
curl_found = $($curl.found.ToString().ToLowerInvariant())
curl_path = "$(ConvertTo-TomlPath $curl.path)"
curl_version = "$($curl.version)"
tar_found = $($tar.found.ToString().ToLowerInvariant())
tar_path = "$(ConvertTo-TomlPath $tar.path)"
tar_version = "$($tar.version)"

[tools.wasi_sdk]
required = true
status = "$(if ([string]::IsNullOrWhiteSpace($WasiSdkRoot)) { "missing" } else { "ready" })"
root = "$(ConvertTo-TomlPath $WasiSdkRoot)"
wasi_root = "$(ConvertTo-TomlPath $wasiRoot)"
clang = "$(ConvertTo-TomlPath $wasiClang)"

[[commands]]
name = "provision_stage2_wasm_host_tooling"
command = "powershell -ExecutionPolicy Bypass -File bootstrap/provision-stage2-wasm-host-tooling.ps1"
exit_code = 0
classification = "tooling_provisioned"
evidence = "Repo-local Ninja $($Ninja.version) is present, bootstrap.toml has [llvm].download-ci-llvm=true, [llvm].ninja=true, and [target.wasm32-wasip1].wasi-root."

[[commands]]
name = "stage2_wasm_host_route_after_ninja"
command = "python x.py build compiler/rustc_span --stage 2 --host wasm32-wasip1 -v"
workdir = "third_party/rust"
exit_code = 1
classification = "stage2_wasm_host_route_requires_cmake_for_llvm_wasm"
evidence = "After repo-local Ninja was on PATH, bootstrap reached `Building LLVM for wasm32-wasip1` and failed at cmake program-not-found. This makes CMake a required route tool unless the LLVM-for-wasm host step is redesigned away."

[[commands]]
name = "stage2_wasm_host_route_after_cmake"
command = "python x.py build compiler/rustc_span --stage 2 --host wasm32-wasip1 -v"
workdir = "third_party/rust"
exit_code = 1
classification = "stage2_wasm_host_route_requires_explicit_wasi_clang_exe_paths"
evidence = "After repo-local CMake was on PATH, CMake started but rejected bootstrap's extensionless wasm32-wasip1-clang and wasm32-wasip1-clang++ paths on Windows. The generated bootstrap.toml now records explicit .exe cc/cxx/ar/ranlib/linker entries."

[[commands]]
name = "stage2_wasm_host_route_after_wasi_exe_paths"
command = "python x.py build compiler/rustc_span --stage 2 --host wasm32-wasip1 -v"
workdir = "third_party/rust"
exit_code = 1
classification = "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
evidence = "With repo-local Ninja, repo-local CMake, [llvm].download-ci-llvm=true, WASI SDK 33.0, and explicit WASI .exe tool paths, the route configured LLVM for wasm32-wasip1 and failed during Ninja compilation of LLVM Support: llvm/include/llvm/ADT/bit.h includes missing ``machine/endian.h`` from the WASI sysroot before rustc_span root artifact emission."
"@

    Write-Utf8NoBom $toolingPath ($content + [Environment]::NewLine)
    return $toolingPath
}

$ninja = Ensure-Ninja
$cmake = Ensure-CMake
$env:PATH = "$($ninja.root);$(Join-Path $cmake.root "bin");$env:PATH"

$wasiSdkRoot = Find-WasiSdkRoot
if ($null -eq $wasiSdkRoot) {
    $wasiProvisioner = Join-RepoPath "bootstrap\provision-wasi-sdk.ps1"
    & powershell -ExecutionPolicy Bypass -File $wasiProvisioner -RepoRoot $RepoRoot | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "bootstrap/provision-wasi-sdk.ps1 failed with exit code $LASTEXITCODE"
    }
    $wasiSdkRoot = Find-WasiSdkRoot
}
if ($null -eq $wasiSdkRoot) {
    throw "WASI SDK root could not be located after provisioning."
}

$env:WASI_SDK_PATH = $wasiSdkRoot
$env:PATH = "$(Join-Path $wasiSdkRoot "bin");$env:PATH"

$bootstrapPath = Set-Stage2BootstrapConfig $wasiSdkRoot (Join-Path $wasiSdkRoot "share\wasi-sysroot")
$toolingPath = Write-ToolingToml $ninja $cmake $wasiSdkRoot $bootstrapPath

[pscustomobject]@{
    status = "ready"
    ninja = [pscustomobject]@{
        path = $ninja.exe
        version = $ninja.version
        archive_sha256 = $ninja.archive_sha256
    }
    cmake = [pscustomobject]@{
        path = $cmake.exe
        version = $cmake.version
        archive_sha256 = $cmake.archive_sha256
    }
    wasi_sdk_root = $wasiSdkRoot
    rust_bootstrap_config = $bootstrapPath
    tooling_record = $toolingPath
    path_prefix = $ninja.root
    stage2_command = "cd third_party/rust; python x.py build compiler/rustc_span --stage 2 --host wasm32-wasip1 -v"
} | ConvertTo-Json -Depth 4
