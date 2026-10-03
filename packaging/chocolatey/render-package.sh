#!/usr/bin/env bash
# Monta o pacote do Chocolatey para VERSION em OUT_DIR, com o SHA256 do .zip
# de Windows daquela versão. Depois: choco pack OUT_DIR/medx.nuspec.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "uso: $0 VERSION SHA256 OUT_DIR" >&2
  exit 2
fi
version=$1
sha=$2
out=$3
src="$(dirname "$0")"

[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "$0: versão inválida: $version" >&2; exit 1; }
[[ "$sha" =~ ^[0-9a-f]{64}$ ]] || { echo "$0: sha256 inválido: $sha" >&2; exit 1; }

mkdir -p "$out/tools"
sed "s/@VERSION@/$version/g" "$src/medx.nuspec.in" > "$out/medx.nuspec"
sed -e "s/@VERSION@/$version/g" -e "s/@SHA256@/$sha/g" \
  "$src/tools/chocolateyinstall.ps1.in" > "$out/tools/chocolateyinstall.ps1"

if grep -q "@[A-Z0-9_]*@" "$out/medx.nuspec" "$out/tools/chocolateyinstall.ps1"; then
  echo "$0: marcador sem preencher no pacote" >&2
  exit 1
fi
