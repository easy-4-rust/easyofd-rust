#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
source_dir=${OFDRW_SOURCE_DIR:?set OFDRW_SOURCE_DIR to the existing reference checkout}
commit=${OFDRW_COMMIT:-84e9eb260ba6fd351ecccf08a01a40305d4af635}
actual=$(git -C "$source_dir" rev-parse HEAD)
[[ "$actual" == "$commit" ]] || { echo "OFDRW SHA mismatch: $actual" >&2; exit 2; }
font="$source_dir/ofdrw-converter/src/test/resources/font_13132_0_edit.ttf"
[[ -f "$font" ]] || { echo 'Missing embedded test font' >&2; exit 2; }
command -v javac >/dev/null
command -v java >/dev/null
output=${1:-"$root/tests/fixtures/upstream_20261014"}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
javac -encoding UTF-8 -d "$work" "$root/tools/upstream/TextFixtures.java"
java -cp "$work" TextFixtures "$work/samples" "$font"
python3 - "$work/samples" "$commit" "$font" <<'PY'
import hashlib,json,pathlib,sys,subprocess
p=pathlib.Path(sys.argv[1])
m={'ofdrw_version':'2.4.0','ofdrw_commit':sys.argv[2],'font_source':'ofdrw-converter/src/test/resources/font_13132_0_edit.ttf','font_sha256':hashlib.sha256(pathlib.Path(sys.argv[3]).read_bytes()).hexdigest(),'font_license_note':'Upstream test asset under repository Apache-2.0; no separate font license supplied. Redistribution review remains required.','generator':'tools/upstream/TextFixtures.java: input construction port, not Java rendering proof','java':subprocess.run(['java','-version'],capture_output=True,text=True).stderr.splitlines()[0],'samples':{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(p.glob('*.ofd'))}}
(p/'manifest.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n')
PY
mkdir -p "$output"
cp "$work/samples/"* "$output/"
