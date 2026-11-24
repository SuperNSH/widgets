#!/usr/bin/env bash
# Publishes the crates to crates.io. Used by .github/workflows/publish.yml.
#   verify         packages both crates and builds them from the packages; needs no token
#   publish VER    uploads the crates that are not on crates.io at VER yet; needs CARGO_REGISTRY_TOKEN
# A published version cannot be replaced or deleted, so a rerun skips what is already there.
set -euo pipefail
cd "$(dirname "$0")/.."

CRATES=(nsh-widgets nsh-widgets-cli)
TRIES=${TRIES:-30}
WAIT=${WAIT:-10}

# Prints the crate's index lines; nothing when crates.io does not have it yet (404).
index() {
  local body status
  body=$(mktemp)
  status=$(curl -sSL --max-time 30 --retry 3 -o "$body" -w '%{http_code}' "https://index.crates.io/${1:0:2}/${1:2:2}/$1")
  case $status in
    200) cat "$body" ;;
    404) ;;
    *) echo "crates.io index answered $status for $1" >&2; rm -f "$body"; return 1 ;;
  esac
  rm -f "$body"
}

# A failed index read must stop the script, not count as "not published".
has() {
  local lines
  lines=$(index "$1") || exit 1
  grep -q "\"vers\":\"$2\"" <<< "$lines"
}

crate_version() {
  cargo metadata --no-deps --format-version 1 --locked | jq -er --arg c "$1" '.packages[] | select(.name == $c) | .version'
}

verify() {
  local crate
  for crate in "${CRATES[@]}"; do
    cargo package --locked -p "$crate"
  done
}

publish() {
  local version=$1 crate v
  for crate in "${CRATES[@]}"; do
    v=$(crate_version "$crate")
    [ "$v" = "$version" ] || { echo "$crate is $v, not $version" >&2; exit 1; }
  done
  for crate in "${CRATES[@]}"; do
    if has "$crate" "$version"; then
      echo "$crate $version is on crates.io already"
      continue
    fi
    # `verify` built the package in a step without the token; --no-verify keeps build scripts away from it.
    cargo publish --locked --no-verify -p "$crate"
    for ((i = 0; ; i++)); do
      has "$crate" "$version" && break
      [ "$i" -lt "$TRIES" ] || { echo "$crate $version did not appear on crates.io" >&2; exit 1; }
      sleep "$WAIT"
    done
    echo "published $crate $version"
  done
}

case ${1:-} in
  verify) verify ;;
  publish) publish "${2:?usage: publish-crates.sh publish VERSION}" ;;
  *) echo "usage: publish-crates.sh verify | publish VERSION" >&2; exit 2 ;;
esac
