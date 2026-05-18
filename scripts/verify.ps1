param()

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $RepoRoot
try {
    & powershell -ExecutionPolicy Bypass -File (Join-Path $RepoRoot "scripts\package.ps1")
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
    $manifestPath = Join-Path $RepoRoot "dist\manifest.json"
    $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    if ($manifest.final_module_artifact.exists -ne $true) {
        throw "dist/manifest.json does not record a promoted final module artifact"
    }
    if ([string]::IsNullOrWhiteSpace([string]$manifest.final_module_artifact.path)) {
        throw "dist/manifest.json final module artifact path is empty"
    }
    if ([string]::IsNullOrWhiteSpace([string]$manifest.final_module_artifact.sha256) -or ([string]$manifest.final_module_artifact.sha256).Length -ne 64) {
        throw "dist/manifest.json final module artifact hash is missing or malformed"
    }
    if ([int64]$manifest.final_module_artifact.size_bytes -le 0) {
        throw "dist/manifest.json final module artifact size is not positive"
    }
    if ($manifest.interface_proof.passed -ne $true) {
        throw "dist/manifest.json interface proof did not pass"
    }
    if ($manifest.runtime_proof_attempted -ne $true -or $manifest.runtime_proof.passed -ne $true) {
        throw "dist/manifest.json runtime proof was not attempted and passed"
    }
    if ([int]$manifest.runtime_proof.exit_code -ne 0) {
        throw "dist/manifest.json runtime proof exit code is not 0"
    }

    $targetDir = Join-Path $RepoRoot ".rouwdi\t"

    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
    $env:CARGO_TARGET_DIR = $targetDir
    $env:CARGO_BUILD_JOBS = "1"

    Write-Host "CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR"
    Write-Host "CARGO_BUILD_JOBS=$env:CARGO_BUILD_JOBS"

    $depsDir = Join-Path $targetDir "debug\deps"
    if (Test-Path -LiteralPath $depsDir -PathType Container) {
        $resolvedTargetDir = (Resolve-Path -LiteralPath $targetDir).Path
        $resolvedDepsDir = (Resolve-Path -LiteralPath $depsDir).Path
        if (-not $resolvedDepsDir.StartsWith($resolvedTargetDir, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to clean linker outputs outside CARGO_TARGET_DIR: $resolvedDepsDir"
        }

        Get-ChildItem -LiteralPath $resolvedDepsDir -File |
            Where-Object { $_.Extension -in @(".exe", ".pdb") } |
            Remove-Item -Force
    }

    & cargo test --workspace
    $exitCode = $LASTEXITCODE
} finally {
    Pop-Location
}

exit $exitCode
