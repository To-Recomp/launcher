#!/usr/bin/env bash
# Builds a throw-away library around a REAL release and the player's own discs, to look at the
# real launcher window with real data. Nothing is copied from the discs; the library lives in
# <dir> (put it under /var/tmp and delete it afterwards: a build takes about 2 GB).
#
#   scripts/visual-real.sh <dir> <release dir> <game id> <platform> <input path>... [-- <program> <args>...]
#
# <release dir> is an installed release folder (e.g. `<library>/tos/release`); it is copied.
# The instance starts as "needs rebuild" with the given files chosen for the inputs, in the
# order the manifest lists them. After `--`, a program and its leading arguments replace the
# release's `prepare` program, e.g. a development `tos-prepare` that prints what a published
# release doesn't yet print:
#
#   -- /var/tmp/tos-prepare-build/tos-prepare --toolkit T --dumps D --compiler "T/../toolchain/zig/zig c++ ..."
#
# Prints the environment that makes the launcher use it (it fetches the real catalog and GitHub's
# answers). Example under X11 or XWayland:
#
#   eval "$(scripts/visual-real.sh /var/tmp/launcher-real ~/.local/share/org.torecomp.launcher/games/tos/release \
#            tos GameCube disc1.iso disc2.iso)"
#   GDK_BACKEND=x11 to-recomp-launcher &
set -euo pipefail

usage() { echo "usage: visual-real.sh <dir> <release dir> <game id> <platform> <input path>... [-- <program> <args>...]" >&2; exit 2; }
[ $# -ge 5 ] || usage
dir=$(realpath -m "$1"); release=$(realpath "$2"); game=$3; platform=$4
shift 4
inputs=()
while [ $# -gt 0 ] && [ "$1" != "--" ]; do inputs+=("$(realpath "$1")"); shift; done
[ "${1:-}" = "--" ] && shift
[ ${#inputs[@]} -gt 0 ] || usage

slug=$(printf '%s' "$platform" | tr 'A-Z' 'a-z' | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g')
lib="$dir/library"
rm -rf "$dir"
mkdir -p "$lib/$game/instances/$slug" "$dir/data" "$dir/config"
cp -r "$release" "$lib/$game/release"

# The ids of the manifest's inputs, in order.
mapfile -t ids < <(python3 -c 'import json,sys; print("\n".join(i["id"] for i in json.load(open(sys.argv[1]))["inputs"]))' "$release/recomplauncher.json")
[ ${#ids[@]} -ge ${#inputs[@]} ] || { echo "more files than the manifest has inputs" >&2; exit 2; }
pairs=""
for i in "${!inputs[@]}"; do pairs+="${pairs:+,}$(printf '"%s": "%s"' "${ids[$i]}" "${inputs[$i]}")"; done

if [ $# -gt 0 ]; then
  program=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["prepare"]["program"][0])' "$release/recomplauncher.json")
  { echo '#!/bin/sh'; printf 'exec'; printf ' %q' "$@"; echo ' "$@"'; } >"$lib/$game/release/$program"
  chmod +x "$lib/$game/release/$program"
fi

cp "$(dirname "$release")/installation.json" "$lib/$game/installation.json"
cat >"$lib/$game/instances/$slug/instance.json" <<JSON
{ "platform": "$platform", "inputs": { $pairs }, "prepared_release": "v0.0.0", "inputs_changed": true }
JSON

cat <<ENV
export TO_RECOMP_LIBRARY='$lib'
export XDG_DATA_HOME='$dir/data'
export XDG_CONFIG_HOME='$dir/config'
ENV
