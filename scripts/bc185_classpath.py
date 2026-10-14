"""Resolve exact local BC1.85 and unchanged OFDRW2.4.0 artifact classpath."""
import pathlib,sys
root=pathlib.Path.home()/'.m2/repository'
files=[]
for name in ['bcprov','bcpkix','bcutil']:
    p=root/'org/bouncycastle'/f'{name}-jdk18on/1.85/{name}-jdk18on-1.85.jar'
    if not p.is_file():raise SystemExit(f'Missing dependency: {p}')
    files.append(p)
build=pathlib.Path(__file__).resolve().parents[1]/'target/ofdrw-fixed-source'
if not (build/'.source-commit').is_file() or (build/'.source-commit').read_text().strip()!='84e9eb260ba6fd351ecccf08a01a40305d4af635':
    raise SystemExit('Build fixed source first with scripts/build_ofdrw_reference.sh')
for name in ['core','gm','sign','gv','reader','pkg']:
    p=build/f'ofdrw-{name}/target/classes'
    if not p.is_dir():raise SystemExit(f'Missing compiled module: {p}')
    files.append(p)
files.append(root/'commons-io/commons-io/2.16.1/commons-io-2.16.1.jar')
print(':'.join(map(str,files)))
