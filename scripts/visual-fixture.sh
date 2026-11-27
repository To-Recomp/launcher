#!/usr/bin/env bash
# Builds a throw-away library for looking at the real launcher window (no game data involved):
# one installed game whose "prepare" is examples/fake_game.rs, with two fake discs, and a catalog
# listing it. A disc whose content contains NOTES makes the fake step print a `release` and two
# `warn` lines (see docs/library.md).
#
#   scripts/visual-fixture.sh <dir> <fake_game binary>
#
# Prints the environment that makes the launcher use it. Example (X11 or XWayland):
#
#   eval "$(scripts/visual-fixture.sh /var/tmp/launcher-visual target/debug/examples/fake_game)"
#   (cd "$FIXTURE_CATALOG_DIR" && python3 -m http.server 8765 --bind 127.0.0.1) &
#   GDK_BACKEND=x11 to-recomp-launcher &
#
# The instance starts as "needs rebuild" (its files were chosen again), so the Home page offers
# "Prepare again": click it and the release and warnings appear under the action status.
set -euo pipefail

dir=${1:?usage: visual-fixture.sh <dir> <fake_game binary>}
fake=${2:?usage: visual-fixture.sh <dir> <fake_game binary>}
dir=$(realpath -m "$dir")
rm -rf "$dir"
lib="$dir/library" release="$lib/tos/release" instance="$lib/tos/instances/gamecube"
mkdir -p "$release" "$instance" "$dir/discs" "$dir/catalog" "$dir/data" "$dir/config"

cp "$fake" "$release/fake-game"
chmod +x "$release/fake-game"
printf 'NOTES disc one' >"$dir/discs/d1.iso"
printf 'disc two' >"$dir/discs/d2.iso"

cat >"$release/recomplauncher.json" <<'JSON'
{
  "format": 1,
  "inputs": [
    { "id": "disc1", "label": "Disc 1", "role": "disc", "formats": [{ "type": "file", "extensions": ["iso"] }] },
    { "id": "disc2", "label": "Disc 2", "role": "disc", "formats": [{ "type": "file", "extensions": ["iso"] }] }
  ],
  "prepare": {
    "program": ["fake-game"],
    "args": ["--prepare", "--data", "{data}", "--disc={input:disc1}", "--disc={input:disc2}"],
    "label": "Building the game",
    "done": ["{data}/built.bin"],
    "keeps_copy": true
  },
  "start": { "program": ["fake-game"], "args": ["--play", "--data", "{data}"] }
}
JSON

printf '{"repository":"To-Recomp/tos-recomp","tag":"v0.1.0","asset":"fake.zip","sha256":"00"}' >"$lib/tos/installation.json"
cat >"$instance/instance.json" <<JSON
{
  "platform": "GameCube",
  "inputs": { "disc1": "$dir/discs/d1.iso", "disc2": "$dir/discs/d2.iso" },
  "prepared_release": "v0.0.9",
  "inputs_changed": true
}
JSON

cat >"$dir/catalog/catalog.json" <<'JSON'
{
  "format": 1,
  "generated": "2026-11-27T00:00:00Z",
  "games": [
    {
      "id": "tos", "name": "Tales of Symphonia", "japan_release_year": 2003,
      "repository": "To-Recomp/tos-recomp",
      "platforms": [{ "platform": "GameCube", "regions": ["France (PAL)"], "state": "playable", "maturity": "alpha", "release_year": 2003 }]
    }
  ]
}
JSON

cat <<ENV
export FIXTURE_CATALOG_DIR='$dir/catalog'
export TO_RECOMP_LIBRARY='$lib'
export TO_RECOMP_CATALOG_URL='http://127.0.0.1:8765/catalog.json'
export TO_RECOMP_GITHUB_API='http://127.0.0.1:1'
export XDG_DATA_HOME='$dir/data'
export XDG_CONFIG_HOME='$dir/config'
ENV
