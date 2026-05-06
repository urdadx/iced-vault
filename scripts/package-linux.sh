#!/usr/bin/env bash
set -euo pipefail

mkdir -p dist
rm -f dist/*.AppImage

cargo packager --release --formats appimage

shopt -s nullglob
artifacts=(dist/*.AppImage)

if [[ ${#artifacts[@]} -eq 0 ]]; then
  printf 'No AppImage artifact was produced.\n' >&2
  exit 1
fi

target="dist/iced-vault-linux-$(uname -m).AppImage"
mv "${artifacts[0]}" "$target"

printf 'Created %s\n' "$target"
