param()

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$MinProductSizeBytes = 1048576

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
    $inside = $false

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

Push-Location $RepoRoot
try {
    $previousCargoTargetDir = $env:CARGO_TARGET_DIR
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue

    try {
        & cargo build -p rouwdi-wasm --target wasm32-wasip1 --release
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build -p rouwdi-wasm --target wasm32-wasip1 --release failed with exit code $LASTEXITCODE"
        }
    } finally {
        if ($null -ne $previousCargoTargetDir) {
            $env:CARGO_TARGET_DIR = $previousCargoTargetDir
        } else {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        }
    }

    $sourceAssembly = Join-Path $RepoRoot "target\wasm32-wasip1\release\rouwdi-assembly.wasm"
    $cdylibStub = Join-Path $RepoRoot "target\wasm32-wasip1\release\rouwdi_wasm.wasm"
    $distDir = Join-Path $RepoRoot "dist"
    $canonicalArtifact = Join-Path $distDir "rouwdi.wasm"
    $manifestPath = Join-Path $distDir "manifest.json"
    $mirExportManifestPath = Join-Path $RepoRoot "bootstrap\mir-payload-export-manifest.toml"

    if (-not (Test-Path -LiteralPath $sourceAssembly -PathType Leaf)) {
        throw "Expected meaningful assembly artifact is missing: $sourceAssembly"
    }
    if (-not (Test-Path -LiteralPath $cdylibStub -PathType Leaf)) {
        throw "Expected cdylib stub artifact is missing: $cdylibStub"
    }

    $sourceIdentity = Get-ArtifactIdentity -Path $sourceAssembly
    $stubIdentity = Get-ArtifactIdentity -Path $cdylibStub

    if ($sourceIdentity.size_bytes -lt $MinProductSizeBytes) {
        throw "Refusing to package suspiciously tiny assembly artifact $($sourceIdentity.path) ($($sourceIdentity.size_bytes) bytes)"
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
    if ($canonicalIdentity.size_bytes -lt $MinProductSizeBytes) {
        throw "Refusing packaged dist/rouwdi.wasm because it is suspiciously tiny ($($canonicalIdentity.size_bytes) bytes)"
    }
    if ($canonicalIdentity.sha256 -eq $stubIdentity.sha256) {
        throw "Refusing packaged dist/rouwdi.wasm because it matches the cdylib stub hash"
    }
    if ($canonicalIdentity.sha256 -ne $sourceIdentity.sha256) {
        throw "Packaged dist/rouwdi.wasm does not match source assembly hash"
    }

    $payloadMetadata = Read-TomlTable -Path $mirExportManifestPath -TableName "exported_payload"
    $payloadPath = [string]$payloadMetadata["path"]
    $payloadSha256 = [string]$payloadMetadata["sha256"]
    $payloadSizeBytes = [int64]$payloadMetadata["size_bytes"]
    $payloadAbsolutePath = Join-Path $RepoRoot $payloadPath
    $payloadExists = Test-Path -LiteralPath $payloadAbsolutePath -PathType Leaf
    $payloadState = "metadata_reference_only"
    $payloadExternal = $false
    $payloadHashVerified = $false

    if ($payloadExists) {
        $actualPayload = Get-ArtifactIdentity -Path $payloadAbsolutePath
        if ($actualPayload.sha256 -ne $payloadSha256) {
            throw "External MIR payload hash mismatch for $payloadPath"
        }
        if ($actualPayload.size_bytes -ne $payloadSizeBytes) {
            throw "External MIR payload size mismatch for $payloadPath"
        }
        $payloadState = "external_hash_verified_payload"
        $payloadExternal = $true
        $payloadHashVerified = $true
    }

    $manifest = [ordered]@{
        schema_version = 1
        package_command = "scripts/package.ps1"
        canonical_artifact_path = "dist/rouwdi.wasm"
        source_build_artifact_path = "target/wasm32-wasip1/release/rouwdi-assembly.wasm"
        artifact = $canonicalIdentity
        source_build_artifact = $sourceIdentity
        rejected_cdylib_stub = [ordered]@{
            path = "target/wasm32-wasip1/release/rouwdi_wasm.wasm"
            size_bytes = $stubIdentity.size_bytes
            sha256 = $stubIdentity.sha256
            rejected_as_product = $true
        }
        mir_payload = [ordered]@{
            state = $payloadState
            embedded = $false
            external = $payloadExternal
            metadata_source_path = "bootstrap/mir-payload-export-manifest.toml"
            path = $payloadPath.Replace("\", "/")
            sha256 = $payloadSha256
            size_bytes = $payloadSizeBytes
            exists = $payloadExists
            hash_verified = $payloadHashVerified
            single_file_product = $false
            note = "The MIR payload bytes are not embedded in dist/rouwdi.wasm; this manifest records only the current external/bootstrap-managed payload reference and verification state."
        }
    }

    $manifestJson = $manifest | ConvertTo-Json -Depth 8
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($manifestPath, $manifestJson, $utf8NoBom)

    Write-Host "packaged dist/rouwdi.wasm"
    Write-Host "source=$($sourceIdentity.path)"
    Write-Host "size=$($canonicalIdentity.size_bytes)"
    Write-Host "sha256=$($canonicalIdentity.sha256)"
    Write-Host "manifest=dist/manifest.json"
    Write-Host "mir_payload_state=$payloadState"
    $exitCode = 0
} catch {
    Write-Error $_
    $exitCode = 1
} finally {
    Pop-Location
}

exit $exitCode
