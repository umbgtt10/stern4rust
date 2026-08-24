# Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the MIT License
# SPDX-License-Identifier: MIT

# just looks for a POSIX `sh` on Windows and there is not reliably one on PATH:
# Git for Windows ships sh.exe without putting it there, and a resolvable
# bash.exe may belong to WSL, a separate toolchain. PowerShell is the one shell
# guaranteed present. Only recipe bodies are shell-interpreted, so this affects
# Windows alone; the Linux and macOS runners use just's default `sh`.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Set by just itself rather than per recipe line, so it reaches whichever shell
# runs the body without needing sh and PowerShell syntax for the same thing.
export RUSTFLAGS := "-D warnings"

# CI fails on drift instead of silently rewriting files nobody is there to
# review; a local run still formats in place.
fmt_mode := if env('CI', '') != '' { '--check' } else { '' }

default:
    @just --list

# Formatting, clippy and tests -- cargo built-ins only, so it works on a fresh
# checkout with none of the house tools installed.
stage1:
    cargo fmt {{fmt_mode}}
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# House rules, grip self-analysis, CRAP, mirrored tests and file risk, run in
# that order. The self-analysis gate is grip4rust's alone: the tool is built
# from this checkout and pointed at core/, so a change that costs the codebase
# testability is caught by the very measure the tool exists to report.
stage2:
    cargo xtask stage2
