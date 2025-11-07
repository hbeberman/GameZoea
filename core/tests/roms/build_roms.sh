#!/bin/bash

set -euo pipefail
shopt -s nullglob

asm_files=( *.asm )

if (( ${#asm_files[@]} == 0 )); then
  echo "No .asm files found in $(pwd)" >&2
  exit 0
fi

for asm_file in "${asm_files[@]}"; do
  base="${asm_file%.asm}"
  obj="${base}.o"
  gb="${base}.gb"

  echo "Assembling ${asm_file} -> ${obj}"
  rgbasm -o "${obj}" "${asm_file}"

  echo "Linking ${obj} -> ${gb}"
  rgblink -o "${gb}" "${obj}"

  rm -f "${obj}"
done
