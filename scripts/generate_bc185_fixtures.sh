#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
source_dir=${OFDRW_SOURCE_DIR:?set existing OFDRW_SOURCE_DIR}
[[ $(git -C "$source_dir" rev-parse HEAD) == 84e9eb260ba6fd351ecccf08a01a40305d4af635 ]] || { echo 'OFDRW SHA mismatch' >&2; exit 2; }
classpath=$(python3 "$root/scripts/bc185_classpath.py")
output=${1:-"$root/target/bc185/java"}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
javac -encoding UTF-8 -cp "$classpath" -d "$work" "$root/tools/upstream/BC185Interop.java"
java -cp "$work:$classpath" BC185Interop generate "$source_dir" "$work/samples"
python3 - "$work/samples" "$source_dir" <<'PYMETA'
import hashlib,json,pathlib,sys,subprocess
p=pathlib.Path(sys.argv[1])
m={'ofdrw_version':'2.4.0','ofdrw_commit':subprocess.check_output(['git','-C',sys.argv[2],'rev-parse','HEAD'],text=True).strip(),'bc_version':'1.85','source_build':'target/ofdrw-fixed-source, compiled fixed Git archive','sm2_user_id':'1234567812345678','test_identity_only':True,'files':{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(p.iterdir())}}
(p/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
PYMETA
mkdir -p "$output"
cp "$work/samples/"* "$output/"
