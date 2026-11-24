#!/bin/bash
# Builds android/nsh-widgets.aar from its Java sources and resources.
# Needs a JDK (javac, jar) and an Android SDK platform jar. The AAR is committed; CI rebuilds it to
# check the sources compile.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
A=$ROOT/android
SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}
PLATFORM=${NSH_WIDGETS_ANDROID_PLATFORM:-$(ls -d "$SDK"/platforms/android-* | sort -V | tail -1)}
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/classes" "$WORK/aar"
javac --release 11 -nowarn -cp "$PLATFORM/android.jar" -d "$WORK/classes" $(find "$A/src" -name '*.java')
(cd "$WORK/classes" && jar cf "$WORK/aar/classes.jar" .)
cp "$A/AndroidManifest.xml" "$WORK/aar/"
cp -R "$A/res" "$WORK/aar/res"
# R.txt: the resources the code looks up (by name, through getIdentifier).
{
  n=0; for f in "$A"/res/drawable/*.xml; do printf 'int drawable %s 0x7f05%04x\n' "$(basename "$f" .xml)" $n; n=$((n+1)); done
  n=0; for id in $(grep -ho '@+id/[a-z_]*' "$A"/res/layout/*.xml | sed 's#@+id/##' | sort -u); do printf 'int id %s 0x7f01%04x\n' "$id" $n; n=$((n+1)); done
  n=0; for f in "$A"/res/layout/*.xml; do printf 'int layout %s 0x7f02%04x\n' "$(basename "$f" .xml)" $n; n=$((n+1)); done
} > "$WORK/aar/R.txt"
rm -f "$A/nsh-widgets.aar"
(cd "$WORK/aar" && find . -exec touch -t 202601010000 {} + && zip -qrX "$A/nsh-widgets.aar" AndroidManifest.xml R.txt classes.jar res)
"$ROOT/scripts/aar-sources.sh" > "$A/nsh-widgets.aar.sources"
echo "built $A/nsh-widgets.aar ($(wc -l < "$WORK/aar/R.txt" | tr -d ' ') resources)"
