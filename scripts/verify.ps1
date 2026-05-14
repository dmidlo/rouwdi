param()

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $RepoRoot
try {
    $targetDir = ".rouwdi/t"

    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
    $env:CARGO_TARGET_DIR = $targetDir

    Write-Host "CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR"
    & cargo test --workspace
    $exitCode = $LASTEXITCODE
} finally {
    Pop-Location
}

exit $exitCode
