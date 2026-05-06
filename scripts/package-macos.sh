#!/usr/bin/env bash
set -euo pipefail

mkdir -p dist/app
rm -rf dist/app/*.app dist/*.app.zip dist/*.dmg

cargo packager --release --formats app,dmg

shopt -s nullglob
apps=(dist/*.app)
dmgs=(dist/*.dmg)

if [[ ${#apps[@]} -eq 0 ]]; then
  printf 'No macOS app bundle was produced.\n' >&2
  exit 1
fi

if [[ ${#dmgs[@]} -eq 0 ]]; then
  printf 'No DMG artifact was produced.\n' >&2
  exit 1
fi

arch="$(uname -m)"
app_target="dist/app/iced-vault-macos-${arch}.app"
dmg_target="dist/iced-vault-macos-${arch}.dmg"
zip_target="dist/iced-vault-macos-${arch}.app.zip"

mv "${apps[0]}" "$app_target"
mv "${dmgs[0]}" "$dmg_target"
ditto -c -k --sequesterRsrc --keepParent "$app_target" "$zip_target"

printf 'Created %s and %s\n' "$dmg_target" "$zip_target"
