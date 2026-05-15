param()

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$MinDevAssemblySizeBytes = 1048576

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

function Write-Utf8NoBom {
    param(
        [string]$Path,
        [string]$Content
    )

    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

Push-Location $RepoRoot
try {
    & cargo build -p rouwdi-wasm --no-default-features --target wasm32-wasip1 --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build -p rouwdi-wasm --no-default-features --target wasm32-wasip1 --release failed with exit code $LASTEXITCODE"
    }

    $distDir = Join-Path $RepoRoot "dist"
    $devArtifact = Join-Path $distDir "rouwdi.dev.wasm"
    $devManifestPath = Join-Path $distDir "manifest.dev.json"
    $sourceAssembly = Join-Path $RepoRoot "target\wasm32-wasip1\release\rouwdi-assembly.wasm"
    $cdylibStub = Join-Path $RepoRoot "target\wasm32-wasip1\release\rouwdi_wasm.wasm"
    $mirExportManifestPath = Join-Path $RepoRoot "bootstrap\mir-payload-export-manifest.toml"

    $payloadMetadata = Read-TomlTable -Path $mirExportManifestPath -TableName "exported_payload"
    $payloadPath = [string]$payloadMetadata["path"]
    $payloadAbsolutePath = Join-Path $RepoRoot $payloadPath
    $payloadExists = Test-Path -LiteralPath $payloadAbsolutePath -PathType Leaf
    $payloadHashVerified = $false

    if ($payloadExists) {
        $actualPayload = Get-ArtifactIdentity -Path $payloadAbsolutePath
        if ($actualPayload.sha256 -ne [string]$payloadMetadata["sha256"]) {
            throw "External MIR payload hash mismatch for dev package: $payloadPath"
        }
        if ($actualPayload.size_bytes -ne [int64]$payloadMetadata["size_bytes"]) {
            throw "External MIR payload size mismatch for dev package: $payloadPath"
        }
        $payloadHashVerified = $true
    }

    $sourceIdentity = Get-ArtifactIdentity -Path $sourceAssembly
    $stubIdentity = Get-ArtifactIdentity -Path $cdylibStub

    if ($sourceIdentity.size_bytes -lt $MinDevAssemblySizeBytes) {
        throw "Refusing suspiciously tiny dev assembly artifact $($sourceIdentity.path)"
    }
    if ($sourceIdentity.sha256 -eq $stubIdentity.sha256) {
        throw "Refusing to package cdylib stub as dev assembly: $($stubIdentity.path)"
    }

    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    Copy-Item -LiteralPath $sourceAssembly -Destination $devArtifact -Force
    $devIdentity = Get-ArtifactIdentity -Path $devArtifact

    $manifest = [ordered]@{
        schema_version = 2
        package_mode = "dev_external_payload"
        package_command = "scripts/package-dev.ps1"
        dev_artifact_path = "dist/rouwdi.dev.wasm"
        single_file_product = $false
        not_product_complete = $true
        reason = "external MIR payload used for faster iteration"
        artifact = $devIdentity
        rejected_cdylib_stub = [ordered]@{
            path = "target/wasm32-wasip1/release/rouwdi_wasm.wasm"
            size_bytes = $stubIdentity.size_bytes
            sha256 = $stubIdentity.sha256
            rejected_as_product = $true
        }
        mir_payload = [ordered]@{
            state = if ($payloadHashVerified) { "external_hash_verified_payload" } else { "metadata_reference_only" }
            embedded = $false
            external = $payloadExists
            path = $payloadPath.Replace("\", "/")
            sha256 = [string]$payloadMetadata["sha256"]
            size_bytes = [int64]$payloadMetadata["size_bytes"]
            hash_verified = $payloadHashVerified
            single_file_product = $false
            not_product_complete = $true
        }
    }

    Write-Utf8NoBom -Path $devManifestPath -Content ($manifest | ConvertTo-Json -Depth 8)

    Write-Host "packaged dev-only dist/rouwdi.dev.wasm"
    Write-Host "package_mode=dev_external_payload"
    Write-Host "single_file_product=false"
    Write-Host "not_product_complete=true"
    $exitCode = 0
} catch {
    Write-Error $_
    $exitCode = 1
} finally {
    Pop-Location
}

exit $exitCode
