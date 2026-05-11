#!/usr/bin/env bash
# check-api-surface.sh — diff the current public API surface against the
# committed 1.0 snapshot. Pattern lifted from agnosys; the cyrius CLI does
# the heavy lifting, this script is a one-line scope+snapshot wrapper.
#
# Cyrius 5.9.13+ supports `--scope=project` (excludes stdlib) and
# `--snapshot=PATH` (alternate snapshot file), so the wrapper stays trivial.
#
# Usage:
#   scripts/check-api-surface.sh              # diff vs. committed snapshot
#   scripts/check-api-surface.sh --update     # regenerate snapshot
#
# Exits non-zero on drift; CI's audit gate consumes the exit code.
set -euo pipefail
cd "$(dirname "$0")/.."
exec cyrius api-surface "$@" \
    --scope=project \
    --snapshot=docs/development/api-surface-1.0.snapshot
