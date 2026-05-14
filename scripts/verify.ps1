param()

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $RepoRoot
try {
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
