# Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the MIT License
# SPDX-License-Identifier: MIT

$ErrorActionPreference = "Stop"
Push-Location (Split-Path $PSScriptRoot -Parent)

function Invoke-Crap4RustGate {
    param(
        [string]$Label,
        [string[]]$Packages,
        [double]$Threshold = 15,
        [string[]]$ExcludePaths = @()
    )

    Write-Host "$Label..." -ForegroundColor Cyan

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path
    $args = @("--manifest-path", $manifestPath)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }
    foreach ($excludePath in $ExcludePaths) {
        $args += @("--exclude-path", $excludePath)
    }
    $args += @("--warn-only", "--threshold", $Threshold.ToString())

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo crap4rust @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $summaryLine = $output | Select-String -Pattern "summary:\s+total_functions=.*crappy_functions=(\d+)"
    if (-not $summaryLine) {
        Write-Host "`nFailed: $Label (could not parse crap4rust summary)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $crappyCount = [int]$summaryLine.Matches[0].Groups[1].Value
    if ($crappyCount -gt 0) {
        Write-Host "`nFailed: $Label ($crappyCount crappy functions detected)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

# A tool that enforces a rule it does not itself satisfy is not worth
# installing. This runs the freshly built binary rather than whatever version
# happens to be installed, so the gate reflects the working tree.
function Invoke-Twin4RustGate {
    param(
        [string]$Label,
        [string[]]$Packages
    )

    Write-Host "$Label..." -ForegroundColor Cyan

    if (-not (Get-Command cargo-twin4rust -ErrorAction SilentlyContinue)) {
        Write-Host "`ncargo-twin4rust is not installed." -ForegroundColor Red
        Write-Host "Install it with: cargo install cargo-twin4rust" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path

    $args = @("twin4rust", "--manifest-path", $manifestPath)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (source files without a mirrored test)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

function Invoke-Stern4RustSelfGate {
    Write-Host "Own rules stern4rust..." -ForegroundColor Cyan

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path

    # Built from source rather than run from whatever is installed. The gate has
    # to judge the tree it is standing in, not the last version that happened to
    # be published.
    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo run --quiet --manifest-path $manifestPath -- --manifest-path $manifestPath 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    # 2 is the tool's own "a rule was broken"; anything else non-zero means it
    # could not run at all, which is a different failure and worth saying so.
    if ($exitCode -eq 2) {
        Write-Host "`nFailed: Own rules stern4rust (a rule was broken)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    if ($exitCode -ne 0) {
        Write-Host "`nFailed: Own rules stern4rust (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

function Invoke-Iceberg4RustGate {
    param(
        [string]$Label,
        [string[]]$Packages,
        [string]$Threshold
    )

    Write-Host "$Label..." -ForegroundColor Cyan

    if (-not (Get-Command cargo-iceberg4rust -ErrorAction SilentlyContinue)) {
        Write-Host "`ncargo-iceberg4rust is not installed." -ForegroundColor Red
        Write-Host "Install it with: cargo install cargo-iceberg4rust" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path

    # The ceiling is passed as a string rather than a [double] so it reaches the
    # CLI unchanged. Interpolating a [double] formats it with the current culture,
    # which emits a comma decimal separator on some locales and fails to parse.
    $args = @("iceberg4rust", "--manifest-path", $manifestPath, "--threshold", $Threshold)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    # 2 is the tool's own "offenders found"; anything else non-zero means it
    # could not run at all.
    if ($exitCode -eq 2) {
        Write-Host "`nFailed: $Label (file at or above the ceiling of $Threshold)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

Invoke-Crap4RustGate "CRAP stern4rust" @("cargo-stern4rust")

# ---------------------------------------------------------------------------
# Mirrored test gate
# ---------------------------------------------------------------------------

Invoke-Twin4RustGate "Mirrored tests stern4rust" @("cargo-stern4rust")

# ---------------------------------------------------------------------------
# File risk gate
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# File risk gate
# ---------------------------------------------------------------------------

Invoke-Iceberg4RustGate "File risk stern4rust" @("cargo-stern4rust") -Threshold "20"

# ---------------------------------------------------------------------------
# Own rules (self-analysis)
#
# A tool that enforces a rule it does not satisfy is not worth installing, so
# every .rs file here carries the header in docs/header.txt.
# ---------------------------------------------------------------------------

Invoke-Stern4RustSelfGate

# ---------------------------------------------------------------------------

Write-Host "`nstern4rust Stage 2 passed!" -ForegroundColor Green
Pop-Location
exit 0
