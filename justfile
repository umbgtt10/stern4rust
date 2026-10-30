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

# The lock file to the newest compatible version of every dependency, while
# the tools are in development. A cargo built-in, so stage 1 still needs none of
# the house tools; it does need the network. Commit what it moves on its own.
sync:
    cargo xtask sync

# Formatting, clippy and tests -- cargo built-ins only, so it works on a fresh
# checkout with none of the house tools installed.
stage1: sync
    cargo fmt {{fmt_mode}}
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Technology-free core, house rules, dependencies, duplication, CRAP, mirrored
# tests and file risk, run in that order. The house rules are this tool's own:
# it is built from this checkout's cli/ and pointed at every member, so a rule
# this checkout breaks is caught by the very tool about to publish it.
stage2:
    cargo xtask stage2
