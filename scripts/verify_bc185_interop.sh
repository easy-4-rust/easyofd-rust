#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
source_dir=${OFDRW_SOURCE_DIR:?set existing OFDRW_SOURCE_DIR}
[[ $(git -C "$source_dir" rev-parse HEAD) == 84e9eb260ba6fd351ecccf08a01a40305d4af635 ]] || { echo 'OFDRW SHA mismatch' >&2; exit 2; }
output=${1:?usage: verify_bc185_interop.sh RUST_OUTPUT}
for name in cert.der message.bin sm2.der ses-v1.der ses-v4.der ses-v5.der public-test.p12; do
    [[ -s "$output/$name" ]] || { echo "Missing output: $name" >&2; exit 2; }
done
classpath=$(python3 "$root/scripts/bc185_classpath.py")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
javac -encoding UTF-8 -cp "$classpath" -d "$work" "$root/tools/upstream/BC185Interop.java"
java -cp "$work:$classpath" BC185Interop verify "$source_dir" "$output"
