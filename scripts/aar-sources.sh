#!/bin/bash
# Prints one SHA-256 over the Android library's sources (Java, resources, manifest): names and contents.
set -euo pipefail
cd "$(dirname "$0")/../android"
sha() { if command -v shasum > /dev/null; then shasum -a 256 "$@"; else sha256sum "$@"; fi; }
find src res AndroidManifest.xml -type f | LC_ALL=C sort | while read -r f; do sha "$f"; done | sha | cut -d' ' -f1
