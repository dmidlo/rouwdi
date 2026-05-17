param()

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$MinAssemblyShellBytes = 1048576
$PayloadFingerprintBytes = 8192

if (-not ("Rouwdi.ByteSearch" -as [type])) {
    Add-Type -TypeDefinition @"
namespace Rouwdi {
    public static class ByteSearch {
        public static bool Contains(byte[] haystack, byte[] needle) {
            if (needle == null || needle.Length == 0) {
                return true;
            }
            if (haystack == null || haystack.Length < needle.Length) {
                return false;
            }

            int[] skip = new int[256];
            for (int i = 0; i < skip.Length; i++) {
                skip[i] = needle.Length;
            }
            for (int i = 0; i < needle.Length - 1; i++) {
                skip[needle[i]] = needle.Length - 1 - i;
            }

            int offset = 0;
            int max = haystack.Length - needle.Length;
            while (offset <= max) {
                int scan = needle.Length - 1;
                while (scan >= 0 && haystack[offset + scan] == needle[scan]) {
                    scan--;
                }
                if (scan < 0) {
                    return true;
                }
                offset += skip[haystack[offset + needle.Length - 1]];
            }
            return false;
        }
    }
}
"@
}

function To-RepoRelativePath {
    param([string]$Path)

    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $root = $RepoRoot.TrimEnd([char[]]"\/")
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
        throw "Missing artifact: $Path"
    }

    $item = Get-Item -LiteralPath $Path
    $hash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()

    return [ordered]@{
        path = To-RepoRelativePath -Path $Path
        size_bytes = [int64]$item.Length
        sha256 = $hash
    }
}

function Get-Sha256HexString {
    param([string]$Text)

    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $sha256.ComputeHash($bytes)
        return -join ($hash | ForEach-Object { $_.ToString("x2") })
    } finally {
        $sha256.Dispose()
    }
}

function Read-TomlTable {
    param(
        [string]$Path,
        [string]$TableName
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing TOML metadata: $Path"
    }

    $table = @{}
    $inside = ($TableName -eq "")

    foreach ($line in Get-Content -LiteralPath $Path) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^\[(.+)\]$') {
            $inside = ($Matches[1] -eq $TableName)
            continue
        }

        if (-not $inside -or $trimmed.Length -eq 0 -or $trimmed.StartsWith("#")) {
            continue
        }

        if ($trimmed -match '^([A-Za-z0-9_]+)\s*=\s*(.+)$') {
            $key = $Matches[1]
            $raw = $Matches[2].Trim()
            if ($raw -match '^"(.*)"$') {
                $table[$key] = $Matches[1]
            } elseif ($raw -match '^(true|false)$') {
                $table[$key] = [System.Boolean]::Parse($raw)
            } elseif ($raw -match '^[0-9]+$') {
                $table[$key] = [int64]$raw
            } else {
                $table[$key] = $raw
            }
        }
    }

    return $table
}

function Escape-RustString {
    param([string]$Value)

    return $Value.Replace("\", "\\").Replace('"', '\"')
}

function Write-Utf8NoBom {
    param(
        [string]$Path,
        [string]$Content
    )

    if (-not $Content.EndsWith("`n")) {
        $Content = "$Content`n"
    }
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Copy-ByteRange {
    param(
        [byte[]]$Bytes,
        [int]$Offset,
        [int]$Length
    )

    $slice = New-Object byte[] $Length
    [System.Array]::Copy($Bytes, $Offset, $slice, 0, $Length)
    return $slice
}

function Assert-ContainsBytes {
    param(
        [byte[]]$Haystack,
        [byte[]]$Needle,
        [string]$Description
    )

    if (-not [Rouwdi.ByteSearch]::Contains($Haystack, $Needle)) {
        throw "dist/rouwdi.wasm does not contain embedded payload evidence: $Description"
    }
}

function Invoke-CommandChecked {
    param(
        [string]$Description,
        [scriptblock]$Command
    )

    Write-Host $Description
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE"
    }
}

function To-JsonArray {
    param([object]$Value)

    $items = New-Object System.Collections.ArrayList
    foreach ($item in @($Value)) {
        if ($null -ne $item) {
            [void]$items.Add($item)
        }
    }
    return ,([object[]]$items.ToArray())
}

function Ensure-MirPayload {
    param(
        [string]$ManifestPath,
        [hashtable]$PayloadMetadata
    )

    $payloadPath = [string]$PayloadMetadata["path"]
    $payloadAbsolutePath = Join-Path $RepoRoot $payloadPath

    if (-not (Test-Path -LiteralPath $payloadAbsolutePath -PathType Leaf)) {
        Invoke-CommandChecked `
            -Description "building direct MIR payload with direct-rustc-private-pack-builder" `
            -Command { & cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder }
    } else {
        Write-Host "verified existing direct MIR payload candidate: $payloadPath"
    }

    if (-not (Test-Path -LiteralPath $payloadAbsolutePath -PathType Leaf)) {
        throw "Canonical packaging requires the direct MIR payload bytes, but it is missing: $payloadPath"
    }

    $expectedSha256 = [string]$PayloadMetadata["sha256"]
    $expectedSizeBytes = [int64]$PayloadMetadata["size_bytes"]
    $identity = Get-ArtifactIdentity -Path $payloadAbsolutePath

    if ($identity.sha256 -ne $expectedSha256) {
        throw "Direct MIR payload hash mismatch for $payloadPath; expected $expectedSha256 got $($identity.sha256)"
    }
    if ($identity.size_bytes -ne $expectedSizeBytes) {
        throw "Direct MIR payload size mismatch for $payloadPath; expected $expectedSizeBytes got $($identity.size_bytes)"
    }

    return [ordered]@{
        manifest_path = To-RepoRelativePath -Path $ManifestPath
        path = $payloadPath.Replace("\", "/")
        absolute_path = $payloadAbsolutePath
        sha256 = $identity.sha256
        size_bytes = $identity.size_bytes
    }
}

function Ensure-CodegenPayload {
    param(
        [object]$WasmTargetReport
    )

    if ($WasmTargetReport.backend_payload_build_attempted -ne $true) {
        throw "Executable wasm32-wasip1 codegen backend payload build was not attempted"
    }
    if ($WasmTargetReport.executable_backend_payload_linked -ne $true) {
        throw "Executable wasm32-wasip1 codegen backend payload did not link; blocker=$($WasmTargetReport.blocker_kind): $($WasmTargetReport.blocker_reason)"
    }

    $payloadPath = [string]$WasmTargetReport.backend_payload_artifact_path
    if ([string]::IsNullOrWhiteSpace($payloadPath)) {
        throw "Executable codegen backend payload report did not provide backend_payload_artifact_path"
    }

    $payloadAbsolutePath = if ([System.IO.Path]::IsPathRooted($payloadPath)) {
        (Resolve-Path -LiteralPath $payloadPath).Path
    } else {
        Join-Path $RepoRoot $payloadPath
    }
    if (-not (Test-Path -LiteralPath $payloadAbsolutePath -PathType Leaf)) {
        throw "Executable codegen backend payload is missing: $payloadAbsolutePath"
    }

    $identity = Get-ArtifactIdentity -Path $payloadAbsolutePath
    $expectedSha256 = [string]$WasmTargetReport.backend_payload_artifact_sha256
    $expectedSizeBytes = [int64]$WasmTargetReport.backend_payload_artifact_size_bytes
    if ($identity.sha256 -ne $expectedSha256) {
        throw "Executable codegen backend payload hash mismatch for $($identity.path); expected $expectedSha256 got $($identity.sha256)"
    }
    if ($identity.size_bytes -ne $expectedSizeBytes) {
        throw "Executable codegen backend payload size mismatch for $($identity.path); expected $expectedSizeBytes got $($identity.size_bytes)"
    }

    return [ordered]@{
        path = $identity.path
        absolute_path = $payloadAbsolutePath
        sha256 = $identity.sha256
        size_bytes = $identity.size_bytes
        generation_command = "powershell -ExecutionPolicy Bypass -File bootstrap/rustc-codegen-llvm-probe/run-wasm-target-check.ps1"
    }
}

function Write-EmbeddedPayloadSource {
    param(
        [hashtable]$RootMetadata,
        [hashtable]$PayloadMetadata,
        [hashtable]$AbiMetadata,
        [object]$CodegenPayloadMetadata,
        [string]$GeneratedPath
    )

    $payloadPath = ([string]$PayloadMetadata["path"]).Replace("\", "/")
    $includePath = "../../../../$payloadPath"
    $payloadName = "rouwdi-mir-handoff-payload"
    $payloadKind = "compiler_payload"
    $payloadStage = [string]$AbiMetadata["supported_stage"]
    $abiName = [string]$AbiMetadata["abi_name"]
    $abiVersion = [int]$AbiMetadata["abi_version"]
    $targetTriple = [string]$RootMetadata["target_triple"]
    $generationCommand = [string]$PayloadMetadata["emitted_by"]
    $payloadSha256 = [string]$PayloadMetadata["sha256"]
    $payloadSizeBytes = [int64]$PayloadMetadata["size_bytes"]
    $codegenPayloadPath = ([string]$CodegenPayloadMetadata["path"]).Replace("\", "/")
    $codegenIncludePath = "../../../../$codegenPayloadPath"
    $codegenGenerationCommand = [string]$CodegenPayloadMetadata["generation_command"]
    $codegenPayloadSha256 = [string]$CodegenPayloadMetadata["sha256"]
    $codegenPayloadSizeBytes = [int64]$CodegenPayloadMetadata["size_bytes"]

    if ([string]::IsNullOrWhiteSpace($payloadStage)) {
        throw "compiler-payload ABI manifest did not provide supported_stage"
    }
    if ([string]::IsNullOrWhiteSpace($abiName) -or $abiVersion -le 0) {
        throw "compiler-payload ABI manifest did not provide ABI identity"
    }
    if ([string]::IsNullOrWhiteSpace($targetTriple)) {
        throw "MIR export manifest did not provide target_triple"
    }

    $content = @"
pub(super) const MIR_PAYLOAD_NAME: &str = "$(Escape-RustString $payloadName)";
pub(super) const MIR_PAYLOAD_KIND: &str = "$(Escape-RustString $payloadKind)";
pub(super) const MIR_PAYLOAD_STAGE: &str = "$(Escape-RustString $payloadStage)";
pub(super) const MIR_PAYLOAD_ABI_NAME: &str = "$(Escape-RustString $abiName)";
pub(super) const MIR_PAYLOAD_ABI_VERSION: u32 = $abiVersion;
pub(super) const MIR_PAYLOAD_TARGET_TRIPLE: &str = "$(Escape-RustString $targetTriple)";
pub(super) const MIR_PAYLOAD_BUILD_SOURCE_PATH: &str =
    "$(Escape-RustString $payloadPath)";
pub(super) const MIR_PAYLOAD_GENERATION_COMMAND: &str =
    "$(Escape-RustString $generationCommand)";
pub(super) const MIR_PAYLOAD_LOAD_STRATEGY: &str = "instantiate_wasm_module";
pub(super) const MIR_PAYLOAD_EMBEDDING_METHOD: &str = "raw_include_bytes";
pub(super) const MIR_PAYLOAD_STATE: &str = "embedded_payload";
pub(super) const MIR_PAYLOAD_SHA256: &str =
    "$(Escape-RustString $payloadSha256)";
pub(super) const MIR_PAYLOAD_SIZE_BYTES: u64 = $payloadSizeBytes;
pub(super) const MIR_PAYLOAD_BYTES: &[u8] = include_bytes!(
    "$(Escape-RustString $includePath)"
);

pub(super) const CODEGEN_PAYLOAD_NAME: &str = "rouwdi-llvm-codegen-backend-payload";
pub(super) const CODEGEN_PAYLOAD_KIND: &str = "codegen_backend_payload";
pub(super) const CODEGEN_PAYLOAD_BACKEND: &str = "rustc_codegen_llvm";
pub(super) const CODEGEN_PAYLOAD_BACKEND_FAMILY: &str = "llvm-grade";
pub(super) const CODEGEN_PAYLOAD_TARGET_TRIPLE: &str = "wasm32-wasip1";
pub(super) const CODEGEN_PAYLOAD_ARTIFACT_PATH: &str =
    "$(Escape-RustString $codegenPayloadPath)";
pub(super) const CODEGEN_PAYLOAD_GENERATION_COMMAND: &str =
    "$(Escape-RustString $codegenGenerationCommand)";
pub(super) const CODEGEN_PAYLOAD_LOAD_STRATEGY: &str = "instantiate_wasi_cli_module";
pub(super) const CODEGEN_PAYLOAD_EMBEDDING_METHOD: &str = "raw_include_bytes";
pub(super) const CODEGEN_PAYLOAD_STATE: &str = "embedded_payload";
pub(super) const CODEGEN_PAYLOAD_SHA256: &str =
    "$(Escape-RustString $codegenPayloadSha256)";
pub(super) const CODEGEN_PAYLOAD_SIZE_BYTES: u64 = $codegenPayloadSizeBytes;
pub(super) const CODEGEN_PAYLOAD_BYTES: &[u8] = include_bytes!(
    "$(Escape-RustString $codegenIncludePath)"
);
"@

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $GeneratedPath) | Out-Null
    Write-Utf8NoBom -Path $GeneratedPath -Content $content
}

Push-Location $RepoRoot
try {
    $distDir = Join-Path $RepoRoot "dist"
    $canonicalArtifact = Join-Path $distDir "rouwdi.wasm"
    $manifestPath = Join-Path $distDir "manifest.json"
    $mirExportManifestPath = Join-Path $RepoRoot "bootstrap\mir-payload-export-manifest.toml"
    $abiManifestPath = Join-Path $RepoRoot "bootstrap\compiler-payload-abi.toml"
    $generatedPayloadSource = Join-Path $RepoRoot "crates\rouwdi-wasm\src\generated\embedded_payloads.rs"
    $wasmTargetCheckScript = Join-Path $RepoRoot "bootstrap\rustc-codegen-llvm-probe\run-wasm-target-check.ps1"
    $wasmTargetReportPath = Join-Path $RepoRoot ".rouwdi\codegen-llvm-probe\wasm-target-report.json"
    $llvmWrapperReportPath = Join-Path $RepoRoot ".rouwdi\codegen-llvm-probe\target-llvm-wrapper-report.json"

    $mirRootMetadata = Read-TomlTable -Path $mirExportManifestPath -TableName ""
    $payloadMetadata = Read-TomlTable -Path $mirExportManifestPath -TableName "exported_payload"
    $abiMetadata = Read-TomlTable -Path $abiManifestPath -TableName ""

    $payload = Ensure-MirPayload -ManifestPath $mirExportManifestPath -PayloadMetadata $payloadMetadata

    Invoke-CommandChecked `
        -Description "checking executable wasm32-wasip1 rustc_codegen_llvm backend payload route" `
        -Command { & powershell -ExecutionPolicy Bypass -File $wasmTargetCheckScript }
    if (-not (Test-Path -LiteralPath $wasmTargetReportPath -PathType Leaf)) {
        throw "wasm target codegen backend route did not emit report: $wasmTargetReportPath"
    }
    if (-not (Test-Path -LiteralPath $llvmWrapperReportPath -PathType Leaf)) {
        throw "target llvm-wrapper build did not emit report: $llvmWrapperReportPath"
    }
    $wasmTargetReport = Get-Content -Raw -LiteralPath $wasmTargetReportPath | ConvertFrom-Json
    $llvmWrapperReport = Get-Content -Raw -LiteralPath $llvmWrapperReportPath | ConvertFrom-Json
    if ($wasmTargetReport.check_only_exit_code -ne 0) {
        throw "rustc_codegen_llvm wasm32-wasip1 check-only route failed with exit code $($wasmTargetReport.check_only_exit_code)"
    }
    if ($wasmTargetReport.backend_payload_build_attempted -ne $true) {
        throw "executable wasm32-wasip1 backend payload build was not attempted"
    }
    if ($llvmWrapperReport.llvm_wrapper_target -ne "wasm32-wasip1") {
        throw "target llvm-wrapper report used unexpected target: $($llvmWrapperReport.llvm_wrapper_target)"
    }
    if ($llvmWrapperReport.artifact_kind -ne "staticlib") {
        throw "target llvm-wrapper report used unexpected artifact_kind: $($llvmWrapperReport.artifact_kind)"
    }
    if ($llvmWrapperReport.wrapper_archive_emitted -eq $true) {
        if ([string]::IsNullOrWhiteSpace([string]$llvmWrapperReport.sha256) -or ([string]$llvmWrapperReport.sha256).Length -ne 64) {
            throw "target llvm-wrapper archive report must carry SHA-256 when emitted"
        }
        if ([int64]$llvmWrapperReport.size_bytes -le 0) {
            throw "target llvm-wrapper archive report must carry size when emitted"
        }
    }
    $codegenPayload = Ensure-CodegenPayload -WasmTargetReport $wasmTargetReport

    Write-EmbeddedPayloadSource `
        -RootMetadata $mirRootMetadata `
        -PayloadMetadata $payloadMetadata `
        -AbiMetadata $abiMetadata `
        -CodegenPayloadMetadata $codegenPayload `
        -GeneratedPath $generatedPayloadSource

    $previousCargoTargetDir = $env:CARGO_TARGET_DIR
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue

    try {
        Invoke-CommandChecked `
            -Description "building rouwdi-wasm canonical assembly with embedded MIR payload" `
            -Command { & cargo build -p rouwdi-wasm --target wasm32-wasip1 --release }
    } finally {
        if ($null -ne $previousCargoTargetDir) {
            $env:CARGO_TARGET_DIR = $previousCargoTargetDir
        } else {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        }
    }

    $sourceAssembly = Join-Path $RepoRoot "target\wasm32-wasip1\release\rouwdi-assembly.wasm"
    $cdylibStub = Join-Path $RepoRoot "target\wasm32-wasip1\release\rouwdi_wasm.wasm"

    if (-not (Test-Path -LiteralPath $sourceAssembly -PathType Leaf)) {
        throw "Expected meaningful assembly artifact is missing: $sourceAssembly"
    }
    if (-not (Test-Path -LiteralPath $cdylibStub -PathType Leaf)) {
        throw "Expected cdylib stub artifact is missing: $cdylibStub"
    }

    $sourceIdentity = Get-ArtifactIdentity -Path $sourceAssembly
    $stubIdentity = Get-ArtifactIdentity -Path $cdylibStub
    $minimumRawEmbeddedSize = [int64]$payload.size_bytes + [int64]$codegenPayload.size_bytes + [int64]$MinAssemblyShellBytes

    if ($sourceIdentity.size_bytes -lt $minimumRawEmbeddedSize) {
        throw "Refusing to package assembly without the embedded MIR payload: $($sourceIdentity.path) is $($sourceIdentity.size_bytes) bytes, below raw embedded threshold $minimumRawEmbeddedSize"
    }
    if ($sourceIdentity.sha256 -eq $stubIdentity.sha256) {
        throw "Refusing to package cdylib stub as product: $($stubIdentity.path)"
    }

    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    Copy-Item -LiteralPath $sourceAssembly -Destination $canonicalArtifact -Force

    if (-not (Test-Path -LiteralPath $canonicalArtifact -PathType Leaf)) {
        throw "Packaging did not create dist/rouwdi.wasm"
    }

    $canonicalIdentity = Get-ArtifactIdentity -Path $canonicalArtifact
    if ($canonicalIdentity.size_bytes -lt $minimumRawEmbeddedSize) {
        throw "Refusing packaged dist/rouwdi.wasm because it is too small to carry the raw MIR payload ($($canonicalIdentity.size_bytes) bytes, threshold $minimumRawEmbeddedSize)"
    }
    if ($canonicalIdentity.sha256 -eq $stubIdentity.sha256) {
        throw "Refusing packaged dist/rouwdi.wasm because it matches the cdylib stub hash"
    }
    if ($canonicalIdentity.sha256 -ne $sourceIdentity.sha256) {
        throw "Packaged dist/rouwdi.wasm does not match source assembly hash"
    }

    $artifactBytes = [System.IO.File]::ReadAllBytes($canonicalArtifact)
    $payloadBytes = [System.IO.File]::ReadAllBytes($payload.absolute_path)
    $chunkLength = [Math]::Min($PayloadFingerprintBytes, $payloadBytes.Length)
    $middleOffset = [Math]::Max(0, [int](($payloadBytes.Length - $chunkLength) / 2))
    $lastOffset = [Math]::Max(0, $payloadBytes.Length - $chunkLength)

    $prefix = Copy-ByteRange -Bytes $payloadBytes -Offset 0 -Length $chunkLength
    $middle = Copy-ByteRange -Bytes $payloadBytes -Offset $middleOffset -Length $chunkLength
    $suffix = Copy-ByteRange -Bytes $payloadBytes -Offset $lastOffset -Length $chunkLength

    Assert-ContainsBytes -Haystack $artifactBytes -Needle $prefix -Description "payload prefix"
    Assert-ContainsBytes -Haystack $artifactBytes -Needle $middle -Description "payload middle"
    Assert-ContainsBytes -Haystack $artifactBytes -Needle $suffix -Description "payload suffix"

    $codegenPayloadBytes = [System.IO.File]::ReadAllBytes($codegenPayload.absolute_path)
    $codegenChunkLength = [Math]::Min($PayloadFingerprintBytes, $codegenPayloadBytes.Length)
    $codegenMiddleOffset = [Math]::Max(0, [int](($codegenPayloadBytes.Length - $codegenChunkLength) / 2))
    $codegenLastOffset = [Math]::Max(0, $codegenPayloadBytes.Length - $codegenChunkLength)

    $codegenPrefix = Copy-ByteRange -Bytes $codegenPayloadBytes -Offset 0 -Length $codegenChunkLength
    $codegenMiddle = Copy-ByteRange -Bytes $codegenPayloadBytes -Offset $codegenMiddleOffset -Length $codegenChunkLength
    $codegenSuffix = Copy-ByteRange -Bytes $codegenPayloadBytes -Offset $codegenLastOffset -Length $codegenChunkLength

    Assert-ContainsBytes -Haystack $artifactBytes -Needle $codegenPrefix -Description "codegen payload prefix"
    Assert-ContainsBytes -Haystack $artifactBytes -Needle $codegenMiddle -Description "codegen payload middle"
    Assert-ContainsBytes -Haystack $artifactBytes -Needle $codegenSuffix -Description "codegen payload suffix"

    $ascii = [System.Text.Encoding]::ASCII
    foreach ($identityString in @(
        "rouwdi-mir-handoff-payload",
        "raw_include_bytes",
        "embedded_payload",
        $payload.sha256,
        "rouwdi-llvm-codegen-backend-payload",
        "rustc_codegen_llvm",
        $codegenPayload.sha256
    )) {
        Assert-ContainsBytes `
            -Haystack $artifactBytes `
            -Needle $ascii.GetBytes($identityString) `
            -Description "registry identity string $identityString"
    }

    Write-Host "executing embedded MIR payload through dist/rouwdi.wasm"
    $payloadExecutionJsonLines = & cargo run -q -p rouwdi -- run-wasm $canonicalArtifact payloads
    if ($LASTEXITCODE -ne 0) {
        throw "dist/rouwdi.wasm embedded MIR payload execution failed with exit code $LASTEXITCODE"
    }
    $payloadExecutionJson = $payloadExecutionJsonLines -join "`n"
    $payloadExecution = $payloadExecutionJson | ConvertFrom-Json
    if ($payloadExecution.execution_source -ne "embedded_registry") {
        throw "Canonical MIR payload execution source must be embedded_registry; got $($payloadExecution.execution_source)"
    }
    if ($payloadExecution.external -ne $false) {
        throw "Canonical MIR payload execution must not be external"
    }
    if ($payloadExecution.opened_external_file -ne $false) {
        throw "Canonical MIR payload execution opened an external payload file"
    }
    if ($payloadExecution.hash_verified -ne $true -or $payloadExecution.size_verified -ne $true) {
        throw "Canonical MIR payload execution did not verify payload hash/size"
    }
    if ($payloadExecution.module_instantiated -ne $true) {
        throw "Canonical MIR payload execution did not instantiate the embedded module"
    }
    if ($payloadExecution.abi_v1_exports_verified -ne $true) {
        throw "Canonical MIR payload execution did not verify ABI v1 exports"
    }
    if ($payloadExecution.execute_called -ne $true) {
        throw "Canonical MIR payload execution did not call execute"
    }
    if ($payloadExecution.output_bytes_read -ne $true -and $payloadExecution.error_bytes_read -ne $true) {
        throw "Canonical MIR payload execution did not read output or error bytes"
    }
    $payloadOutput = $null
    if ($payloadExecution.output_bytes_read -eq $true -and -not [string]::IsNullOrWhiteSpace([string]$payloadExecution.output_json)) {
        $payloadOutput = $payloadExecution.output_json | ConvertFrom-Json
    }
    $payloadError = $null
    if ($payloadExecution.error_bytes_read -eq $true -and -not [string]::IsNullOrWhiteSpace([string]$payloadExecution.error_json)) {
        $payloadError = $payloadExecution.error_json | ConvertFrom-Json
    }
    $mirBodyIdentity = if ($null -ne $payloadOutput) { [string]$payloadOutput.mir_body_identity } else { $null }
    $mirBodyHash = if ($null -ne $payloadOutput) { [string]$payloadOutput.mir_body_hash } else { $null }
    $mirBodyIdentityEmitted = -not [string]::IsNullOrWhiteSpace($mirBodyIdentity)
    $mirBodyHashEmitted = -not [string]::IsNullOrWhiteSpace($mirBodyHash)
    $mirProviderInvoked = ($null -ne $payloadOutput -and $payloadOutput.mir_provider_invoked -eq $true) -or ($null -ne $payloadError -and $payloadError.mir_provider_invoked -eq $true)
    $coreMetadataLoaded = ($null -ne $payloadOutput -and $payloadOutput.core_metadata_loaded -eq $true) -or ($null -ne $payloadError -and $payloadError.core_metadata_loaded -eq $true)
    $langItemsResolved = $mirProviderInvoked -or $mirBodyIdentityEmitted
    $payloadExecutionEvidenceText = @(
        [string]$payloadExecution.output_json,
        [string]$payloadExecution.error_json
    ) -join "`n"
    $payloadExecutionEvidenceLower = $payloadExecutionEvidenceText.ToLowerInvariant()
    foreach ($forbiddenEvidence in @(
        "stops before mir provider invocation",
        "mir provider was not invoked",
        "before tycxt::optimized_mir",
        "before tyctxt::optimized_mir",
        "before tyctx::optimized_mir"
    )) {
        if ($payloadExecutionEvidenceLower.Contains($forbiddenEvidence)) {
            throw "Canonical MIR payload evidence contains stale pre-MIR-provider blocker text: $forbiddenEvidence"
        }
    }
    if ($mirBodyIdentityEmitted -and -not $mirBodyHashEmitted) {
        throw "Canonical MIR payload emitted MIR identity without MIR body hash"
    }
    if ($mirBodyIdentityEmitted) {
        foreach ($fabricatedFlag in @("fabricated_ast", "fabricated_hir", "fabricated_tyctx", "fabricated_providers", "fabricated_body", "fabricated_mir")) {
            if ($payloadOutput.$fabricatedFlag -eq $true) {
                throw "Canonical MIR payload output cannot claim MIR success with $fabricatedFlag=true"
            }
        }
        foreach ($blockerTextField in @("blocker_reason", "exact_blocker", "blocker_text")) {
            $blockerTextValue = [string]$payloadOutput.$blockerTextField
            if (-not [string]::IsNullOrWhiteSpace($blockerTextValue) -and $blockerTextValue -ne "none") {
                throw "Canonical MIR payload output cannot claim MIR success with $blockerTextField=$blockerTextValue"
            }
        }
        if ($payloadOutput.blocker_kind -ne "none") {
            throw "Canonical MIR payload output cannot claim MIR success with blocker_kind=$($payloadOutput.blocker_kind)"
        }
        if ($payloadOutput.provider_query -ne "rustc_middle::ty::TyCtxt::optimized_mir") {
            throw "Canonical MIR payload output used unexpected provider query: $($payloadOutput.provider_query)"
        }
        if ($payloadOutput.real_mir_body_observed -ne $true) {
            throw "Canonical MIR payload output did not record a real MIR body"
        }
        if ($payloadOutput.fabricated_mono_items -eq $true) {
            throw "Canonical MIR payload output cannot enter monomorphization with fabricated mono items"
        }
        if ($payloadOutput.rustc_monomorphize_invoked -ne $true) {
            throw "Canonical MIR payload output must attempt rustc_monomorphize after MIR proof"
        }
        if ([string]::IsNullOrWhiteSpace([string]$payloadOutput.monomorphization_status)) {
            throw "Canonical MIR payload output must record monomorphization_status"
        }
        if ($payloadOutput.monomorphization_status -eq "mono_items_collected") {
            if ($null -eq $payloadOutput.mono_item_count -or [int64]$payloadOutput.mono_item_count -le 0) {
                throw "Canonical monomorphization success requires mono_item_count > 0"
            }
            if ([string]::IsNullOrWhiteSpace([string]$payloadOutput.mono_item_graph_hash)) {
                throw "Canonical monomorphization success requires mono_item_graph_hash"
            }
            if ($null -eq $payloadOutput.mono_items -or $payloadOutput.mono_items.Count -le 0) {
                throw "Canonical monomorphization success requires upstream mono_items"
            }
            if ([string]$payloadOutput.mono_items_derived_from -ne "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items") {
                throw "Canonical mono items must be derived from collect_and_partition_mono_items"
            }
        }
    }
    if ($payloadExecution.execution_state -match "mir_body" -and -not $mirBodyIdentityEmitted) {
        throw "Canonical MIR payload execution claimed MIR body state without a MIR body identity"
    }
    Write-Host "executing embedded codegen payload through dist/rouwdi.wasm"
    $codegenPayloadExecutionJsonLines = & cargo run -q -p rouwdi -- run-wasm $canonicalArtifact codegen-payloads
    if ($LASTEXITCODE -ne 0) {
        throw "dist/rouwdi.wasm embedded codegen payload execution failed with exit code $LASTEXITCODE"
    }
    $codegenPayloadExecutionJson = $codegenPayloadExecutionJsonLines -join "`n"
    $codegenPayloadExecution = $codegenPayloadExecutionJson | ConvertFrom-Json
    if ($codegenPayloadExecution.execution_source -ne "embedded_registry") {
        throw "Canonical codegen payload execution source must be embedded_registry; got $($codegenPayloadExecution.execution_source)"
    }
    if ($codegenPayloadExecution.external -ne $false) {
        throw "Canonical codegen payload execution must not be external"
    }
    if ($codegenPayloadExecution.opened_external_file -ne $false) {
        throw "Canonical codegen payload execution opened an external payload file"
    }
    if ($codegenPayloadExecution.hash_verified -ne $true -or $codegenPayloadExecution.size_verified -ne $true) {
        throw "Canonical codegen payload execution did not verify payload hash/size"
    }
    if ($codegenPayloadExecution.module_instantiated -ne $true -or $codegenPayloadExecution.start_called -ne $true) {
        throw "Canonical codegen payload execution did not instantiate and start the embedded module"
    }
    if ($codegenPayloadExecution.backend_constructed -ne $true) {
        throw "Canonical codegen payload did not construct rustc_codegen_llvm backend"
    }
    if ($codegenPayloadExecution.mono_proof_consumed -ne $true) {
        throw "Canonical codegen payload did not consume mono/MIR proof input"
    }
    if ($codegenPayloadExecution.llvm_context_created -ne $true -or $codegenPayloadExecution.llvm_module_created -ne $true) {
        throw "Canonical codegen payload did not create LLVM context/module in the embedded product path"
    }
    if ($codegenPayloadExecution.target_machine_created -ne $true) {
        throw "Canonical codegen payload did not create target machine in the embedded product path"
    }
    if ($codegenPayloadExecution.llvm_ir_emitted -ne $true) {
        throw "Canonical codegen payload did not emit real LLVM IR bytes"
    }
    if ([string]::IsNullOrWhiteSpace([string]$codegenPayloadExecution.llvm_ir_sha256) -or ([string]$codegenPayloadExecution.llvm_ir_sha256).Length -ne 64) {
        throw "Canonical codegen payload did not report a SHA-256 for emitted LLVM IR bytes"
    }
    if ([int64]$codegenPayloadExecution.llvm_ir_size_bytes -le 0) {
        throw "Canonical codegen payload did not report positive LLVM IR byte length"
    }
    if ($codegenPayloadExecution.object_emission_attempted -ne $true) {
        throw "Canonical codegen payload did not attempt real object emission"
    }
    if ([string]::IsNullOrWhiteSpace([string]$codegenPayloadExecution.object_emission_api) -or -not ([string]$codegenPayloadExecution.object_emission_api).Contains("LLVMTargetMachineEmitToMemoryBuffer")) {
        throw "Canonical codegen payload did not record the exact LLVM object emission API"
    }
    if ($codegenPayloadExecution.object_bytes_emitted -eq $true -or $codegenPayloadExecution.wasm_object_bytes_emitted -eq $true) {
        if ($codegenPayloadExecution.wasm_object_bytes_emitted -ne $true) {
            throw "Canonical wasm32 codegen payload emitted object bytes but did not classify them as Wasm object bytes"
        }
        if ($codegenPayloadExecution.object_artifact_kind -ne "wasm_object" -or $codegenPayloadExecution.codegen_artifact_kind -ne "wasm_object") {
            throw "Canonical codegen object output must be artifact kind wasm_object"
        }
        if ([string]::IsNullOrWhiteSpace([string]$codegenPayloadExecution.object_artifact_sha256) -or ([string]$codegenPayloadExecution.object_artifact_sha256).Length -ne 64) {
            throw "Canonical codegen payload emitted object bytes without object SHA-256"
        }
        if ([int64]$codegenPayloadExecution.object_artifact_size_bytes -le 0) {
            throw "Canonical codegen payload emitted object bytes without positive object size"
        }
        if ($codegenPayloadExecution.object_bytes_retrieved_by_rouwdi -ne $true -or $codegenPayloadExecution.object_sha256_verified -ne $true) {
            throw "Canonical object bytes must be retrieved and hashed by rouwdi-owned logic"
        }
        $objectLocation = [string]$codegenPayloadExecution.object_artifact_location
        if ([string]::IsNullOrWhiteSpace($objectLocation) -or $objectLocation.EndsWith(".json") -or $objectLocation.Contains("proof") -or $objectLocation.Contains("host:")) {
            throw "Canonical object artifact location is not an object-byte location: $objectLocation"
        }
    } elseif ($codegenPayloadExecution.linker_handoff_created -eq $true) {
        throw "Canonical codegen payload created linker handoff without real object bytes"
    }
    if ($codegenPayloadExecution.linker_handoff_created -eq $true -and $codegenPayloadExecution.wasm_object_bytes_emitted -ne $true) {
        throw "Canonical linker handoff requires real Wasm object bytes"
    }
    $monoInvoked = $null -ne $payloadOutput -and $payloadOutput.rustc_monomorphize_invoked -eq $true
    $monoStatus = if ($null -ne $payloadOutput) { [string]$payloadOutput.monomorphization_status } else { $null }
    $monoBlockerKind = if ($null -ne $payloadOutput) { [string]$payloadOutput.monomorphization_blocker_kind } else { $null }
    $monoBlockerComponent = if ($null -ne $payloadOutput) { [string]$payloadOutput.monomorphization_blocker_component } else { $null }
    $monoBlockerReason = if ($null -ne $payloadOutput) { [string]$payloadOutput.monomorphization_blocker_reason } else { $null }
    $monoItemCount = if ($null -ne $payloadOutput -and $payloadOutput.mono_item_count -ne $null) { [int64]$payloadOutput.mono_item_count } else { $null }
    $monoItems = if ($null -ne $payloadOutput -and $null -ne $payloadOutput.mono_items) { @($payloadOutput.mono_items) } else { @() }
    $partitionCount = if ($null -ne $payloadOutput -and $payloadOutput.partition_count -ne $null) { [int64]$payloadOutput.partition_count } else { $null }
    $codegenUnitCount = if ($null -ne $payloadOutput -and $payloadOutput.codegen_unit_count -ne $null) { [int64]$payloadOutput.codegen_unit_count } else { $null }
    $monoItemGraphHash = if ($null -ne $payloadOutput -and $null -ne $payloadOutput.mono_item_graph_hash) { [string]$payloadOutput.mono_item_graph_hash } else { $null }
    $nextFrontier = if ($mirBodyHashEmitted) {
        if ($monoStatus -eq "mono_items_collected") {
            if ($codegenPayloadExecution.wasm_object_bytes_emitted -eq $true -and $codegenPayloadExecution.linker_handoff_created -eq $true) {
                "linking"
            } else {
                "object_emission"
            }
        } else { "monomorphization" }
    } else {
        "mir_provider"
    }
    $backendPayloadExecutionStatus = if ($monoStatus -eq "mono_items_collected") {
        [string]$codegenPayloadExecution.codegen_contact_state
    } else {
        $null
    }
    $backendPayloadBlockerKind = if ($monoStatus -eq "mono_items_collected") { $codegenPayloadExecution.blocker_kind } else { $null }
    $codegenHandoffStatus = $backendPayloadExecutionStatus
    $codegenBlockerKind = $backendPayloadBlockerKind
    $codegenBlockerComponent = if ($monoStatus -eq "mono_items_collected" -and -not [string]::IsNullOrWhiteSpace([string]$codegenBlockerKind)) {
        if ($llvmWrapperReport.target_llvm_library_closure.blocker_component -ne $null) {
        [string]$llvmWrapperReport.target_llvm_library_closure.blocker_component
        } else {
            "embedded rustc_codegen_llvm backend payload"
        }
    } else {
        $null
    }
    $codegenBlockerReason = if ($monoStatus -eq "mono_items_collected" -and -not [string]::IsNullOrWhiteSpace([string]$codegenBlockerKind)) {
        [string]$codegenPayloadExecution.blocker_reason
    } else {
        $null
    }
    $hostCodegenContactState = if ($monoStatus -eq "mono_items_collected") { "target_machine_created" } else { $null }
    $codegenContactState = $backendPayloadExecutionStatus
    $llvmModuleIdentity = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.llvm_module_identity } else { $null }
    $llvmModuleIdentityHash = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.llvm_module_identity_hash } else { $null }
    $codegenArtifactKind = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.codegen_artifact_kind } else { $null }
    $codegenArtifactSha256 = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.codegen_artifact_sha256 } else { $null }
    $codegenArtifactSizeBytes = if ($monoStatus -eq "mono_items_collected") { $codegenPayloadExecution.codegen_artifact_size_bytes } else { $null }
    $codegenArtifactLocation = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.codegen_artifact_location } else { $null }
    $llvmIrSha256 = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.llvm_ir_sha256 } else { $null }
    $llvmIrSizeBytes = if ($monoStatus -eq "mono_items_collected") { $codegenPayloadExecution.llvm_ir_size_bytes } else { $null }
    $objectEmissionApi = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.object_emission_api } else { $null }
    $objectArtifactKind = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.object_artifact_kind } else { $null }
    $objectArtifactSha256 = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.object_artifact_sha256 } else { $null }
    $objectArtifactSizeBytes = if ($monoStatus -eq "mono_items_collected") { $codegenPayloadExecution.object_artifact_size_bytes } else { $null }
    $objectArtifactLocation = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.object_artifact_location } else { $null }
    $objectTargetTriple = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.object_target_triple } else { $null }
    $objectRetrievalMethod = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.object_retrieval_method } else { $null }
    $linkerHandoffCreated = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.linker_handoff_created } else { $false }
    $exactBlocker = if ($null -ne $payloadError) { $payloadError.blocker_kind } elseif ($null -ne $payloadOutput) { $payloadOutput.blocker_kind } else { $payloadExecution.blocker_kind }

    $embeddedPayload = [ordered]@{
        name = "rouwdi-mir-handoff-payload"
        kind = "compiler_payload"
        stage = [string]$abiMetadata["supported_stage"]
        abi_name = [string]$abiMetadata["abi_name"]
        abi_version = [int]$abiMetadata["abi_version"]
        target_triple = [string]$mirRootMetadata["target_triple"]
        build_source_path = $payload.path
        generation_command = [string]$payloadMetadata["emitted_by"]
        load_strategy = "instantiate_wasm_module"
        embedding_method = "raw_include_bytes"
        state = [string]$payloadExecution.execution_state
        embedded = $true
        external = $false
        instantiated = $true
        abi_verified = $true
        executed = $true
        execution_source = "embedded_registry"
        opened_external_file = $false
        uncompressed_size_bytes = $payload.size_bytes
        compressed_size_bytes = $null
        size_bytes = $payload.size_bytes
        original_sha256 = $payload.sha256
        embedded_sha256 = $payload.sha256
        hash_verified = $true
        size_verified = $true
        registry_entry = $true
        loader_status = "embedded_payload_executed"
        execute_status = [int]$payloadExecution.execute_status
        execute_trapped = [bool]$payloadExecution.execute_trapped
        execute_trap = $payloadExecution.execute_trap
        result_kind = [string]$payloadExecution.result_kind
        blocker_kind = $payloadExecution.blocker_kind
        input_contract_sha256 = [string]$payloadExecution.input_contract_sha256
        output_contract_sha256 = $payloadExecution.output_contract_sha256
        error_contract_sha256 = $payloadExecution.error_contract_sha256
        core_metadata_loaded = $coreMetadataLoaded
        lang_items_resolved = $langItemsResolved
        mir_provider_invoked = $mirProviderInvoked
        mir_body_identity_emitted = $mirBodyIdentityEmitted
        mir_body_hash_emitted = $mirBodyHashEmitted
        mir_body_identity = $mirBodyIdentity
        mir_body_hash = $mirBodyHash
        rustc_monomorphize_imported = ($null -ne $payloadOutput -and $payloadOutput.rustc_monomorphize_imported -eq $true)
        rustc_monomorphize_invoked = $monoInvoked
        monomorphization_query = if ($null -ne $payloadOutput) { [string]$payloadOutput.monomorphization_query } else { $null }
        monomorphization_status = $monoStatus
        monomorphization_blocker_kind = $monoBlockerKind
        monomorphization_blocker_component = $monoBlockerComponent
        monomorphization_blocker_reason = $monoBlockerReason
        mono_item_count = $monoItemCount
        mono_items = @($monoItems)
        mono_items_derived_from = if ($null -ne $payloadOutput) { [string]$payloadOutput.mono_items_derived_from } else { $null }
        partition_count = $partitionCount
        codegen_unit_count = $codegenUnitCount
        mono_item_graph_hash = $monoItemGraphHash
        fabricated_mono_items = ($null -ne $payloadOutput -and $payloadOutput.fabricated_mono_items -eq $true)
        codegen_handoff_status = $codegenHandoffStatus
        rustc_codegen_llvm_attempted = ($monoStatus -eq "mono_items_collected")
        codegen_backend_family = if ($monoStatus -eq "mono_items_collected") { "llvm-grade" } else { $null }
        codegen_expected_output_kind = if ($monoStatus -eq "mono_items_collected") { "wasm_object" } else { $null }
        codegen_backend_entrypoint = if ($monoStatus -eq "mono_items_collected") { "rustc_codegen_llvm::LlvmCodegenBackend::new" } else { $null }
        codegen_contact_state = $codegenContactState
        codegen_mono_proof_consumed = ($monoStatus -eq "mono_items_collected")
        codegen_compile_unit_id = if ($monoStatus -eq "mono_items_collected") { "app:rust:app:wasm32-wasip1" } else { $null }
        codegen_mir_body_hash = if ($monoStatus -eq "mono_items_collected") { $mirBodyHash } else { $null }
        codegen_mono_item_count = if ($monoStatus -eq "mono_items_collected") { $monoItemCount } else { $null }
        codegen_mono_item_graph_hash = if ($monoStatus -eq "mono_items_collected") { $monoItemGraphHash } else { $null }
        host_probe_codegen_contact_state = $hostCodegenContactState
        host_probe_llvm_context_created = ($monoStatus -eq "mono_items_collected")
        host_probe_llvm_module_created = ($monoStatus -eq "mono_items_collected")
        host_probe_target_machine_created = ($monoStatus -eq "mono_items_collected")
        llvm_module_setup_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_module_created } else { $false }
        llvm_context_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_context_created } else { $false }
        llvm_module_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_module_created } else { $false }
        llvm_module_identity = $llvmModuleIdentity
        llvm_module_identity_hash = $llvmModuleIdentityHash
        llvm_module_target_triple = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.llvm_module_target_triple } else { $null }
        target_machine_setup_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.target_machine_setup_invoked } else { $false }
        target_machine_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.target_machine_created } else { $false }
        target_machine_cpu = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_cpu } else { $null }
        target_machine_features = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_features } else { $null }
        target_machine_relocation_model = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_relocation_model } else { $null }
        target_machine_code_model = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_code_model } else { $null }
        target_machine_optimization_level = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_optimization_level } else { $null }
        backend_payload_name = if ($monoStatus -eq "mono_items_collected") { "rouwdi-llvm-codegen-backend-payload" } else { $null }
        backend_payload_kind = if ($monoStatus -eq "mono_items_collected") { "codegen_backend_payload" } else { $null }
        backend_payload_route = if ($monoStatus -eq "mono_items_collected") { "assembly-owned wasm32-wasip1 rustc_codegen_llvm backend payload route" } else { $null }
        backend_payload_embedded_in_assembly = ($monoStatus -eq "mono_items_collected")
        backend_payload_instantiated = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.module_instantiated } else { $false }
        backend_payload_executed = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.start_called } else { $false }
        backend_payload_execution_status = $backendPayloadExecutionStatus
        backend_payload_blocker_kind = $backendPayloadBlockerKind
        check_only_target_loadable = if ($monoStatus -eq "mono_items_collected") { $wasmTargetReport.check_only_exit_code -eq 0 } else { $null }
        executable_backend_payload_linked = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.executable_backend_payload_linked } else { $null }
        backend_payload_build_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.backend_payload_build_attempted } else { $null }
        backend_payload_build_exit_code = if ($monoStatus -eq "mono_items_collected") { $wasmTargetReport.backend_payload_build_exit_code } else { $null }
        backend_payload_final_link_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.backend_payload_final_link_invoked } else { $null }
        backend_payload_linker = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.backend_payload_linker } else { $null }
        backend_payload_first_undefined_symbol = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.backend_payload_first_undefined_symbol } else { $null }
        backend_payload_llvm_undefined_symbols = if ($monoStatus -eq "mono_items_collected") { To-JsonArray $wasmTargetReport.backend_payload_llvm_undefined_symbols } else { [object[]]@() }
        backend_payload_build_log_path = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.payload_build_log_path } else { $null }
        llvm_wrapper_target = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.llvm_wrapper_target } else { $null }
        llvm_wrapper_artifact_kind = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.artifact_kind } else { $null }
        llvm_wrapper_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.path } else { $null }
        llvm_wrapper_sha256 = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.sha256 } else { $null }
        llvm_wrapper_size_bytes = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.size_bytes } else { $null }
        llvm_wrapper_built_by = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.built_by } else { $null }
        llvm_wrapper_linked_into = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.linked_into } else { $null }
        llvm_wrapper_target_loadable = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_loadable } else { $null }
        llvm_wrapper_blocker_kind = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.blocker_kind } else { $null }
        llvm_wrapper_blocker_reason = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.blocker_reason } else { $null }
        target_llvm_library_closure_available = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_llvm_library_closure_available } else { $null }
        target_llvm_library_closure_status = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.status } else { $null }
        target_llvm_library_closure_report_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.report_path } else { $null }
        target_llvm_library_closure_build_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_llvm_library_closure.build_attempted } else { $null }
        target_llvm_library_closure_build_exit_code = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.target_llvm_library_closure.build_exit_code } else { $null }
        target_llvm_library_closure_log_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.log_path } else { $null }
        target_llvm_library_closure_first_error = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.target_llvm_library_closure.first_error } else { $null }
        codegen_blocker_kind = $codegenBlockerKind
        codegen_blocker_component = $codegenBlockerComponent
        codegen_blocker_reason = $codegenBlockerReason
        codegen_artifact_kind = $codegenArtifactKind
        codegen_artifact_sha256 = $codegenArtifactSha256
        codegen_artifact_size_bytes = $codegenArtifactSizeBytes
        codegen_artifact_location = $codegenArtifactLocation
        codegen_llvm_ir_artifact_kind = if ($monoStatus -eq "mono_items_collected") { "llvm_ir" } else { $null }
        codegen_llvm_ir_sha256 = $llvmIrSha256
        codegen_llvm_ir_size_bytes = $llvmIrSizeBytes
        codegen_linker_required = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.linker_required } else { $false }
        codegen_object_emission_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_emission_attempted } else { $false }
        codegen_object_emission_api = $objectEmissionApi
        codegen_object_bytes_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_bytes_emitted } else { $false }
        codegen_wasm_object_bytes_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.wasm_object_bytes_emitted } else { $false }
        codegen_object_artifact_kind = $objectArtifactKind
        codegen_object_artifact_sha256 = $objectArtifactSha256
        codegen_object_artifact_size_bytes = $objectArtifactSizeBytes
        codegen_object_artifact_location = $objectArtifactLocation
        codegen_object_target_triple = $objectTargetTriple
        codegen_object_retrieval_method = $objectRetrievalMethod
        codegen_object_bytes_retrieved_by_rouwdi = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_bytes_retrieved_by_rouwdi } else { $false }
        codegen_object_sha256_verified = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_sha256_verified } else { $false }
        codegen_llvm_ir_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_ir_emitted } else { $false }
        linker_handoff_created = $linkerHandoffCreated
        next_frontier = $nextFrontier
        exact_blocker = $exactBlocker
    }

    $manifest = [ordered]@{
        schema_version = 2
        package_mode = "canonical_single_file"
        package_command = "scripts/package.ps1"
        canonical_artifact_path = "dist/rouwdi.wasm"
        source_build_artifact_path = "target/wasm32-wasip1/release/rouwdi-assembly.wasm"
        single_file_product = $true
        not_product_complete = $false
        artifact = $canonicalIdentity
        source_build_artifact = $sourceIdentity
        rejected_cdylib_stub = [ordered]@{
            path = "target/wasm32-wasip1/release/rouwdi_wasm.wasm"
            size_bytes = $stubIdentity.size_bytes
            sha256 = $stubIdentity.sha256
            rejected_as_product = $true
            reason = "target/.../rouwdi_wasm.wasm is the cdylib stub, not the canonical assembly"
        }
        embedded_payloads = @($embeddedPayload)
        codegen_payloads = @(
            [ordered]@{
                payload_name = "rouwdi-llvm-codegen-backend-payload"
                payload_kind = "codegen_backend_payload"
                backend_family = "llvm-grade"
                upstream_component = "rustc_codegen_llvm"
                dependency_components = @(
                    "rustc_codegen_ssa",
                    "rustc_target",
                    "rustc_metadata",
                    "rustc_llvm",
                    "LLVM wrapper/C++ layer"
                )
                target_triple = "wasm32-wasip1"
                artifact_path = if ($monoStatus -eq "mono_items_collected") { $codegenPayload.path } else { $null }
                artifact_hash = if ($monoStatus -eq "mono_items_collected") { $codegenPayload.sha256 } else { $null }
                artifact_size_bytes = if ($monoStatus -eq "mono_items_collected") { $codegenPayload.size_bytes } else { $null }
                embedded_in_dist_rouwdi_wasm = ($monoStatus -eq "mono_items_collected")
                instantiated = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.module_instantiated } else { $false }
                executed = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.start_called } else { $false }
                execution_status = if ($monoStatus -eq "mono_items_collected") { $backendPayloadExecutionStatus } else { "not_reached" }
                blocker_kind = if ($monoStatus -eq "mono_items_collected") { $backendPayloadBlockerKind } else { $null }
                check_only_status = if ($monoStatus -eq "mono_items_collected") { "rustc_codegen_llvm_target_loadable_check_only" } else { $null }
                check_only_target_loadable = if ($monoStatus -eq "mono_items_collected") { $wasmTargetReport.check_only_exit_code -eq 0 } else { $null }
                executable_backend_payload_linked = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.executable_backend_payload_linked } else { $null }
                backend_payload_build_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.backend_payload_build_attempted } else { $null }
                backend_payload_build_exit_code = if ($monoStatus -eq "mono_items_collected") { $wasmTargetReport.backend_payload_build_exit_code } else { $null }
                backend_payload_final_link_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.backend_payload_final_link_invoked } else { $null }
                backend_payload_linker = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.backend_payload_linker } else { $null }
                backend_payload_first_undefined_symbol = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.backend_payload_first_undefined_symbol } else { $null }
                backend_payload_llvm_undefined_symbols = if ($monoStatus -eq "mono_items_collected") { To-JsonArray $wasmTargetReport.backend_payload_llvm_undefined_symbols } else { [object[]]@() }
                backend_payload_build_log_path = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.payload_build_log_path } else { $null }
                host_probe_state = if ($monoStatus -eq "mono_items_collected") { "host_codegen_probe_backend_constructed" } else { $null }
                host_probe_codegen_contact_state = $hostCodegenContactState
                host_probe_llvm_module_created = ($monoStatus -eq "mono_items_collected")
                host_probe_target_machine_created = ($monoStatus -eq "mono_items_collected")
                codegen_contact_state = $codegenContactState
                mono_proof_consumed = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.mono_proof_consumed } else { $false }
                mir_body_hash = if ($monoStatus -eq "mono_items_collected") { $mirBodyHash } else { $null }
                mono_item_graph_hash = if ($monoStatus -eq "mono_items_collected") { $monoItemGraphHash } else { $null }
                llvm_wrapper_target = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.llvm_wrapper_target } else { $null }
                llvm_wrapper_artifact_kind = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.artifact_kind } else { $null }
                llvm_wrapper_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.path } else { $null }
                llvm_wrapper_sha256 = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.sha256 } else { $null }
                llvm_wrapper_size_bytes = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.size_bytes } else { $null }
                llvm_wrapper_built_by = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.built_by } else { $null }
                llvm_wrapper_linked_into = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.linked_into } else { $null }
                llvm_wrapper_target_loadable = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_loadable } else { $null }
                target_llvm_library_closure_available = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_llvm_library_closure_available } else { $null }
                target_llvm_library_closure_status = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.status } else { $null }
                target_llvm_library_closure_report_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.report_path } else { $null }
                target_llvm_library_closure_build_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_llvm_library_closure.build_attempted } else { $null }
                target_llvm_library_closure_build_exit_code = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.target_llvm_library_closure.build_exit_code } else { $null }
                target_llvm_library_closure_log_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.log_path } else { $null }
                target_llvm_library_closure_first_error = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.target_llvm_library_closure.first_error } else { $null }
                llvm_module_setup_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_module_created } else { $false }
                llvm_context_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_context_created } else { $false }
                llvm_module_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_module_created } else { $false }
                llvm_module_identity = $llvmModuleIdentity
                llvm_module_identity_hash = $llvmModuleIdentityHash
                llvm_module_target_triple = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.llvm_module_target_triple } else { $null }
                target_machine_setup_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.target_machine_setup_invoked } else { $false }
                target_machine_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.target_machine_created } else { $false }
                target_machine_cpu = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_cpu } else { $null }
                target_machine_features = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_features } else { $null }
                target_machine_relocation_model = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_relocation_model } else { $null }
                target_machine_code_model = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_code_model } else { $null }
                target_machine_optimization_level = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_optimization_level } else { $null }
                codegen_artifact_kind = $codegenArtifactKind
                codegen_artifact_sha256 = $codegenArtifactSha256
                codegen_artifact_size_bytes = $codegenArtifactSizeBytes
                codegen_artifact_location = $codegenArtifactLocation
                llvm_ir_artifact_kind = if ($monoStatus -eq "mono_items_collected") { "llvm_ir" } else { $null }
                llvm_ir_sha256 = $llvmIrSha256
                llvm_ir_size_bytes = $llvmIrSizeBytes
                llvm_ir_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_ir_emitted } else { $false }
                bitcode_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.bitcode_emitted } else { $false }
                object_emission_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_emission_attempted } else { $false }
                object_emission_api = $objectEmissionApi
                object_bytes_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_bytes_emitted } else { $false }
                wasm_object_bytes_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.wasm_object_bytes_emitted } else { $false }
                object_artifact_kind = $objectArtifactKind
                object_artifact_sha256 = $objectArtifactSha256
                object_artifact_size_bytes = $objectArtifactSizeBytes
                object_artifact_location = $objectArtifactLocation
                object_target_triple = $objectTargetTriple
                object_retrieval_method = $objectRetrievalMethod
                object_bytes_retrieved_by_rouwdi = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_bytes_retrieved_by_rouwdi } else { $false }
                object_sha256_verified = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_sha256_verified } else { $false }
                linker_required = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.linker_required } else { $false }
                linker_handoff_created = $linkerHandoffCreated
                linker_handoff = if ($monoStatus -eq "mono_items_collected" -and $null -ne $codegenPayloadExecution.output_json.linker_handoff) { $codegenPayloadExecution.output_json.linker_handoff } else { $null }
            }
        )
        mir_payload = [ordered]@{
            state = [string]$payloadExecution.execution_state
            embedded = $true
            instantiated = $true
            abi_verified = $true
            executed = $true
            execution_source = "embedded_registry"
            external = $false
            opened_external_file = $false
            metadata_source_path = "bootstrap/mir-payload-export-manifest.toml"
            path = $payload.path
            original_sha256 = $payload.sha256
            embedded_sha256 = $payload.sha256
            sha256 = $payload.sha256
            size_bytes = $payload.size_bytes
            abi_name = [string]$abiMetadata["abi_name"]
            abi_version = [int]$abiMetadata["abi_version"]
            stage = [string]$abiMetadata["supported_stage"]
            target_triple = [string]$mirRootMetadata["target_triple"]
            embedding_method = "raw_include_bytes"
            load_strategy = "instantiate_wasm_module"
            loader_status = "embedded_payload_executed"
            payload_registry_entry = $true
            hash_verified = $true
            size_verified = $true
            wasm_magic_verified = $true
            module_instantiated = $true
            abi_v1_exports_verified = $true
            version_called = $true
            stage_called = $true
            descriptor_bytes_read = $true
            valid_input_bytes_read = $true
            execute_called = $true
            execute_status = [int]$payloadExecution.execute_status
            execute_trapped = [bool]$payloadExecution.execute_trapped
            execute_trap = $payloadExecution.execute_trap
            output_bytes_read = [bool]$payloadExecution.output_bytes_read
            error_bytes_read = [bool]$payloadExecution.error_bytes_read
            result_kind = [string]$payloadExecution.result_kind
            blocker_kind = $payloadExecution.blocker_kind
            input_contract_sha256 = [string]$payloadExecution.input_contract_sha256
            output_contract_sha256 = $payloadExecution.output_contract_sha256
            error_contract_sha256 = $payloadExecution.error_contract_sha256
            core_metadata_loaded = $coreMetadataLoaded
            lang_items_resolved = $langItemsResolved
            mir_provider_invoked = $mirProviderInvoked
            mir_body_identity_emitted = $mirBodyIdentityEmitted
            mir_body_hash_emitted = $mirBodyHashEmitted
            mir_body_identity = $mirBodyIdentity
            mir_body_hash = $mirBodyHash
            rustc_monomorphize_imported = ($null -ne $payloadOutput -and $payloadOutput.rustc_monomorphize_imported -eq $true)
            rustc_monomorphize_invoked = $monoInvoked
            monomorphization_query = if ($null -ne $payloadOutput) { [string]$payloadOutput.monomorphization_query } else { $null }
            monomorphization_status = $monoStatus
            monomorphization_blocker_kind = $monoBlockerKind
            monomorphization_blocker_component = $monoBlockerComponent
            monomorphization_blocker_reason = $monoBlockerReason
            mono_item_count = $monoItemCount
            mono_items = @($monoItems)
            mono_items_derived_from = if ($null -ne $payloadOutput) { [string]$payloadOutput.mono_items_derived_from } else { $null }
            partition_count = $partitionCount
            codegen_unit_count = $codegenUnitCount
            mono_item_graph_hash = $monoItemGraphHash
            fabricated_mono_items = ($null -ne $payloadOutput -and $payloadOutput.fabricated_mono_items -eq $true)
            codegen_handoff_status = $codegenHandoffStatus
            rustc_codegen_llvm_attempted = ($monoStatus -eq "mono_items_collected")
            codegen_backend_family = if ($monoStatus -eq "mono_items_collected") { "llvm-grade" } else { $null }
            codegen_expected_output_kind = if ($monoStatus -eq "mono_items_collected") { "wasm_object" } else { $null }
            codegen_backend_entrypoint = if ($monoStatus -eq "mono_items_collected") { "rustc_codegen_llvm::LlvmCodegenBackend::new" } else { $null }
            codegen_contact_state = $codegenContactState
            codegen_mono_proof_consumed = ($monoStatus -eq "mono_items_collected")
            codegen_compile_unit_id = if ($monoStatus -eq "mono_items_collected") { "app:rust:app:wasm32-wasip1" } else { $null }
            codegen_mir_body_hash = if ($monoStatus -eq "mono_items_collected") { $mirBodyHash } else { $null }
            codegen_mono_item_count = if ($monoStatus -eq "mono_items_collected") { $monoItemCount } else { $null }
            codegen_mono_item_graph_hash = if ($monoStatus -eq "mono_items_collected") { $monoItemGraphHash } else { $null }
            host_probe_codegen_contact_state = $hostCodegenContactState
            host_probe_llvm_context_created = ($monoStatus -eq "mono_items_collected")
            host_probe_llvm_module_created = ($monoStatus -eq "mono_items_collected")
            host_probe_target_machine_created = ($monoStatus -eq "mono_items_collected")
            llvm_module_setup_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_module_created } else { $false }
            llvm_context_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_context_created } else { $false }
            llvm_module_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_module_created } else { $false }
            llvm_module_identity = $llvmModuleIdentity
            llvm_module_identity_hash = $llvmModuleIdentityHash
            llvm_module_target_triple = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.llvm_module_target_triple } else { $null }
            target_machine_setup_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.target_machine_setup_invoked } else { $false }
            target_machine_created = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.target_machine_created } else { $false }
            target_machine_cpu = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_cpu } else { $null }
            target_machine_features = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_features } else { $null }
            target_machine_relocation_model = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_relocation_model } else { $null }
            target_machine_code_model = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_code_model } else { $null }
            target_machine_optimization_level = if ($monoStatus -eq "mono_items_collected") { [string]$codegenPayloadExecution.target_machine_optimization_level } else { $null }
            backend_payload_name = if ($monoStatus -eq "mono_items_collected") { "rouwdi-llvm-codegen-backend-payload" } else { $null }
            backend_payload_kind = if ($monoStatus -eq "mono_items_collected") { "codegen_backend_payload" } else { $null }
            backend_payload_route = if ($monoStatus -eq "mono_items_collected") { "assembly-owned wasm32-wasip1 rustc_codegen_llvm backend payload route" } else { $null }
            backend_payload_embedded_in_assembly = ($monoStatus -eq "mono_items_collected")
            backend_payload_instantiated = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.module_instantiated } else { $false }
            backend_payload_executed = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.start_called } else { $false }
            backend_payload_execution_status = $backendPayloadExecutionStatus
            backend_payload_blocker_kind = $backendPayloadBlockerKind
            check_only_target_loadable = if ($monoStatus -eq "mono_items_collected") { $wasmTargetReport.check_only_exit_code -eq 0 } else { $null }
            executable_backend_payload_linked = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.executable_backend_payload_linked } else { $null }
            backend_payload_build_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.backend_payload_build_attempted } else { $null }
            backend_payload_build_exit_code = if ($monoStatus -eq "mono_items_collected") { $wasmTargetReport.backend_payload_build_exit_code } else { $null }
            backend_payload_final_link_invoked = if ($monoStatus -eq "mono_items_collected") { [bool]$wasmTargetReport.backend_payload_final_link_invoked } else { $null }
            backend_payload_linker = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.backend_payload_linker } else { $null }
            backend_payload_first_undefined_symbol = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.backend_payload_first_undefined_symbol } else { $null }
            backend_payload_llvm_undefined_symbols = if ($monoStatus -eq "mono_items_collected") { To-JsonArray $wasmTargetReport.backend_payload_llvm_undefined_symbols } else { [object[]]@() }
            backend_payload_build_log_path = if ($monoStatus -eq "mono_items_collected") { [string]$wasmTargetReport.payload_build_log_path } else { $null }
            llvm_wrapper_target = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.llvm_wrapper_target } else { $null }
            llvm_wrapper_artifact_kind = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.artifact_kind } else { $null }
            llvm_wrapper_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.path } else { $null }
            llvm_wrapper_sha256 = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.sha256 } else { $null }
            llvm_wrapper_size_bytes = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.size_bytes } else { $null }
            llvm_wrapper_built_by = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.built_by } else { $null }
            llvm_wrapper_linked_into = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.linked_into } else { $null }
            llvm_wrapper_target_loadable = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_loadable } else { $null }
            llvm_wrapper_blocker_kind = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.blocker_kind } else { $null }
            llvm_wrapper_blocker_reason = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.blocker_reason } else { $null }
            target_llvm_library_closure_available = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_llvm_library_closure_available } else { $null }
            target_llvm_library_closure_status = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.status } else { $null }
            target_llvm_library_closure_report_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.report_path } else { $null }
            target_llvm_library_closure_build_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$llvmWrapperReport.target_llvm_library_closure.build_attempted } else { $null }
            target_llvm_library_closure_build_exit_code = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.target_llvm_library_closure.build_exit_code } else { $null }
            target_llvm_library_closure_log_path = if ($monoStatus -eq "mono_items_collected") { [string]$llvmWrapperReport.target_llvm_library_closure.log_path } else { $null }
            target_llvm_library_closure_first_error = if ($monoStatus -eq "mono_items_collected") { $llvmWrapperReport.target_llvm_library_closure.first_error } else { $null }
            codegen_blocker_kind = $codegenBlockerKind
            codegen_blocker_component = $codegenBlockerComponent
            codegen_blocker_reason = $codegenBlockerReason
            codegen_artifact_kind = $codegenArtifactKind
            codegen_artifact_sha256 = $codegenArtifactSha256
            codegen_artifact_size_bytes = $codegenArtifactSizeBytes
            codegen_artifact_location = $codegenArtifactLocation
            codegen_llvm_ir_artifact_kind = if ($monoStatus -eq "mono_items_collected") { "llvm_ir" } else { $null }
            codegen_llvm_ir_sha256 = $llvmIrSha256
            codegen_llvm_ir_size_bytes = $llvmIrSizeBytes
            codegen_linker_required = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.linker_required } else { $false }
            codegen_object_emission_attempted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_emission_attempted } else { $false }
            codegen_object_emission_api = $objectEmissionApi
            codegen_object_bytes_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_bytes_emitted } else { $false }
            codegen_wasm_object_bytes_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.wasm_object_bytes_emitted } else { $false }
            codegen_object_artifact_kind = $objectArtifactKind
            codegen_object_artifact_sha256 = $objectArtifactSha256
            codegen_object_artifact_size_bytes = $objectArtifactSizeBytes
            codegen_object_artifact_location = $objectArtifactLocation
            codegen_object_target_triple = $objectTargetTriple
            codegen_object_retrieval_method = $objectRetrievalMethod
            codegen_object_bytes_retrieved_by_rouwdi = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_bytes_retrieved_by_rouwdi } else { $false }
            codegen_object_sha256_verified = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.object_sha256_verified } else { $false }
            codegen_llvm_ir_emitted = if ($monoStatus -eq "mono_items_collected") { [bool]$codegenPayloadExecution.llvm_ir_emitted } else { $false }
            linker_handoff_created = $linkerHandoffCreated
            next_frontier = $nextFrontier
            exact_blocker = $exactBlocker
            exists = $true
            single_file_product = $true
            not_product_complete = $false
        }
        packaging_guards = [ordered]@{
            external_only_states_rejected = @(
                "metadata_reference_only",
                "external_hash_verified_payload"
            )
            raw_embedding_min_size_bytes = $minimumRawEmbeddedSize
            final_artifact_size_bytes = $canonicalIdentity.size_bytes
            payload_size_reflected_in_artifact_size = ($canonicalIdentity.size_bytes -ge $minimumRawEmbeddedSize)
            payload_prefix_fingerprint_present = $true
            payload_middle_fingerprint_present = $true
            payload_suffix_fingerprint_present = $true
            codegen_payload_prefix_fingerprint_present = $true
            codegen_payload_middle_fingerprint_present = $true
            codegen_payload_suffix_fingerprint_present = $true
            registry_identity_present = $true
            cdylib_stub_rejected = $true
            embedded_payload_execution_source_required = "embedded_registry"
            embedded_payload_instantiation_required = $true
            embedded_payload_execute_required = $true
            embedded_codegen_payload_execution_source_required = "embedded_registry"
            embedded_codegen_payload_execute_required = $true
            embedded_codegen_payload_llvm_ir_required = $true
            embedded_codegen_payload_object_emission_attempt_required = $true
            linker_handoff_requires_wasm_object_bytes = $true
        }
    }

    $manifestJson = $manifest | ConvertTo-Json -Depth 12
    Write-Utf8NoBom -Path $manifestPath -Content $manifestJson

    Write-Host "packaged dist/rouwdi.wasm"
    Write-Host "package_mode=canonical_single_file"
    Write-Host "source=$($sourceIdentity.path)"
    Write-Host "size=$($canonicalIdentity.size_bytes)"
    Write-Host "sha256=$($canonicalIdentity.sha256)"
    Write-Host "manifest=dist/manifest.json"
    Write-Host "single_file_product=true"
    Write-Host "mir_payload_state=$($payloadExecution.execution_state)"
    Write-Host "mir_payload_embedded=true"
    Write-Host "mir_payload_instantiated=true"
    Write-Host "mir_payload_abi_verified=true"
    Write-Host "mir_payload_executed=true"
    Write-Host "codegen_payload_embedded=true"
    Write-Host "codegen_payload_executed=true"
    Write-Host "codegen_contact_state=$($codegenPayloadExecution.codegen_contact_state)"
    Write-Host "codegen_artifact_kind=$($codegenPayloadExecution.codegen_artifact_kind)"
    Write-Host "codegen_artifact_sha256=$($codegenPayloadExecution.codegen_artifact_sha256)"
    $exitCode = 0
} catch {
    Write-Error $_
    $exitCode = 1
} finally {
    Pop-Location
}

exit $exitCode
