# Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the MIT License
# SPDX-License-Identifier: MIT

$ErrorActionPreference = "Stop"
Push-Location (Split-Path $PSScriptRoot -Parent)

function Invoke-Step {
    param([string]$Label, [scriptblock]$Command)
    Write-Host "$Label..." -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed: $Label (exit code $LASTEXITCODE)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

$env:RUSTFLAGS = "-D warnings"

# ---------------------------------------------------------------------------
# Format + Lint
# ---------------------------------------------------------------------------

Invoke-Step "Formatting" { cargo fmt }

Invoke-Step "Clippy" { cargo clippy --all-targets -- -D warnings }

# ---------------------------------------------------------------------------
# Integration Tests
# ---------------------------------------------------------------------------

Invoke-Step "stern4rust tests" { cargo test }

Write-Host "`nstern4rust validation tests passed!" -ForegroundColor Green
Pop-Location
exit 0
