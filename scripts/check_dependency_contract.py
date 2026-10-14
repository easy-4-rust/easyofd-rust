"""拒绝已淘汰依赖和审计豁免，覆盖生产与Fuzz锁文件。"""
import pathlib
import re
import sys

root = pathlib.Path(__file__).resolve().parents[1]
problems = []
for relative in (sys.argv[1:] or ["Cargo.lock", "fuzz/Cargo.lock"]):
    for block in (root / relative).read_text().split("[[package]]")[1:]:
        name = re.search(r'^name = "([^"]+)"', block, re.M)
        version = re.search(r'^version = "([^"]+)"', block, re.M)
        if name is None or version is None:
            continue
        package, release = name[1], version[1]
        if package in {"ttf-parser", "fontdue", "proc-macro-error", "proc-macro-error2"}:
            problems.append(f"{relative}: forbidden {package}@{release}")
        if package == "lopdf" and tuple(map(int, release.split(".")[:2])) < (0, 42):
            problems.append(f"{relative}: vulnerable {package}@{release}")
for relative in (".cargo/audit.toml", "deny.toml"):
    content = re.sub(r"#[^\n]*", "", (root / relative).read_text())
    if re.search(r'ignore\s*=\s*\[\s*[^\s\]]', content):
        problems.append(f"{relative}: audit exemptions are forbidden")
if problems:
    print("\n".join(problems), file=sys.stderr)
    sys.exit(1)
print("Dependency contract passed: root/Fuzz lockfiles and audit policies")
