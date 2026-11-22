#!/usr/bin/env bash
# Builds the source RPM from the current checkout (what COPR and a local mock build use).
#   packaging/rpm/build-srpm.sh [outdir]      # default: ./rpmbuild-out
set -euo pipefail

root=$(git rev-parse --show-toplevel)
outdir=$(realpath -m "${1:-$root/rpmbuild-out}")
spec="$root/packaging/rpm/to-recomp-launcher.spec"
version=$(rpmspec -q --qf '%{version}\n' --srpm "$spec" | head -n1)

mkdir -p "$outdir/SOURCES"
git -C "$root" archive --prefix="launcher-$version/" -o "$outdir/SOURCES/launcher-$version.tar.gz" HEAD
rpmbuild -bs "$spec" \
  --define "_topdir $outdir" \
  --define "_sourcedir $outdir/SOURCES" \
  --define "_srcrpmdir $outdir"
