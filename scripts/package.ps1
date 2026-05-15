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
        throw "dist/rouwdi.wasm does not contain embedded MIR payload evidence: $Description"
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

function Write-EmbeddedPayloadSource {
    param(
        [hashtable]$RootMetadata,
        [hashtable]$PayloadMetadata,
        [hashtable]$AbiMetadata,
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

    $mirRootMetadata = Read-TomlTable -Path $mirExportManifestPath -TableName ""
    $payloadMetadata = Read-TomlTable -Path $mirExportManifestPath -TableName "exported_payload"
    $abiMetadata = Read-TomlTable -Path $abiManifestPath -TableName ""

    $payload = Ensure-MirPayload -ManifestPath $mirExportManifestPath -PayloadMetadata $payloadMetadata
    Write-EmbeddedPayloadSource `
        -RootMetadata $mirRootMetadata `
        -PayloadMetadata $payloadMetadata `
        -AbiMetadata $abiMetadata `
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
    $minimumRawEmbeddedSize = [int64]$payload.size_bytes + [int64]$MinAssemblyShellBytes

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

    $ascii = [System.Text.Encoding]::ASCII
    foreach ($identityString in @(
        "rouwdi-mir-handoff-payload",
        "raw_include_bytes",
        "embedded_payload",
        $payload.sha256
    )) {
        Assert-ContainsBytes `
            -Haystack $artifactBytes `
            -Needle $ascii.GetBytes($identityString) `
            -Description "registry identity string $identityString"
    }

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
        state = "embedded_payload_hash_verified"
        embedded = $true
        external = $false
        uncompressed_size_bytes = $payload.size_bytes
        compressed_size_bytes = $null
        size_bytes = $payload.size_bytes
        original_sha256 = $payload.sha256
        embedded_sha256 = $payload.sha256
        hash_verified = $true
        size_verified = $true
        registry_entry = $true
        loader_status = "embedded_bytes_available"
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
        mir_payload = [ordered]@{
            state = "embedded_payload"
            embedded = $true
            external = $false
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
            loader_status = "embedded_payload_hash_verified"
            payload_registry_entry = $true
            hash_verified = $true
            size_verified = $true
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
            registry_identity_present = $true
            cdylib_stub_rejected = $true
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
    Write-Host "mir_payload_state=embedded_payload"
    Write-Host "mir_payload_embedded=true"
    $exitCode = 0
} catch {
    Write-Error $_
    $exitCode = 1
} finally {
    Pop-Location
}

exit $exitCode
