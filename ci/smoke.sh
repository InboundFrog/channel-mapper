#!/usr/bin/env bash
# CLI smoke test, run by CI after the tests (RUST_EXTRA in .github/repo-tools.conf):
# list -> apply -> list with the saved map -> list falling back to the saved map.
# Uses the release build from CI's build step; the output in tvmap-out/ is
# uploaded as an artifact.
set -euo pipefail
cd "$(dirname "$0")/.."

fixture=tests/fixtures/Channel_list_T-KTMDEUC-1410.1.zip
out=tvmap-out/${fixture##*/}
tvmap() { cargo run --quiet --release --locked -- "$@"; }

tvmap list "$fixture" --map maps/au/wa/perth.toml
tvmap apply "$fixture" --map maps/au/wa/perth.toml -o "$out" --save-map tvmap-out/channel-map.toml
tvmap list "$out" --map tvmap-out/channel-map.toml
# No --map: falls back to tvmap-out/channel-map.toml.
tvmap list "$out"
