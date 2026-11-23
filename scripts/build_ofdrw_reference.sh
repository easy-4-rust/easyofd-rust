#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
source_dir=${OFDRW_SOURCE_DIR:?set existing fixed OFDRW_SOURCE_DIR}
commit=84e9eb260ba6fd351ecccf08a01a40305d4af635
[[ $(git -C "$source_dir" rev-parse HEAD) == "$commit" ]] || { echo 'OFDRW SHA mismatch' >&2; exit 2; }
build="$root/target/ofdrw-fixed-source"
if [[ -e "$build" && -f "$build/.source-commit" && $(cat "$build/.source-commit") != "$commit" ]]; then
    echo 'Existing reference build belongs to another commit; choose a new output directory' >&2; exit 2
fi
mkdir -p "$build"
git -C "$source_dir" archive "$commit" | tar -x -C "$build"
# 写入完成标记前，必须实际构建成功；不改变外部参考仓库。
mvn -q -f "$build/pom.xml" -pl ofdrw-converter,ofdrw-sign -am package -DskipTests -Dmaven.javadoc.skip=true
printf '%s\n' "$commit" > "$build/.source-commit"
