#!/bin/bash
# Rebuilds the Android AAR and checks the committed one matches its sources: the same files, the same
# resources, R.txt and manifest byte for byte, and the same classes (class bytes vary with the JDK).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
AAR=$ROOT/android/nsh-widgets.aar
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cp "$AAR" "$WORK/committed.aar"
"$ROOT/scripts/build-aar.sh" >/dev/null
mkdir -p "$WORK/a" "$WORK/b"
(cd "$WORK/a" && unzip -q "$WORK/committed.aar" && mkdir c && cd c && unzip -q ../classes.jar)
(cd "$WORK/b" && unzip -q "$AAR" && mkdir c && cd c && unzip -q ../classes.jar)
cp "$WORK/committed.aar" "$AAR"
list() { (cd "$1" && find . -type f ! -name classes.jar ! -path './c/META-INF/*' | sort); }
if ! diff <(list "$WORK/a") <(list "$WORK/b") >/dev/null; then
  echo "android/nsh-widgets.aar is stale: its files differ from the sources. Run scripts/build-aar.sh."; diff <(list "$WORK/a") <(list "$WORK/b") || true; exit 1
fi
if ! diff -r -x c -x classes.jar "$WORK/a" "$WORK/b" >/dev/null; then
  echo "android/nsh-widgets.aar is stale: resources, R.txt or the manifest differ. Run scripts/build-aar.sh."; exit 1
fi
echo "android/nsh-widgets.aar matches its sources"
