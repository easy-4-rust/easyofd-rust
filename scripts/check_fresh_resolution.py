"""从无锁文件消费者解析依赖，检查问题包及声明的Rust1.88兼容性。"""
import json
import pathlib
import subprocess
import sys
import tempfile

root = pathlib.Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix="easyofd-consumer-") as directory:
    project = pathlib.Path(directory)
    (project / "src").mkdir()
    (project / "src/lib.rs").write_text("pub fn smoke() {}\n")
    (project / "Cargo.toml").write_text(
        '[package]\nname = "easyofd-dependency-consumer"\nversion = "0.0.0"\n'
        'edition = "2024"\nrust-version = "1.88"\n'
        '[workspace]\nresolver = "3"\n[dependencies]\neasyofd = { path = '
        + json.dumps(str(root / "crates/easyofd")) + ' }\n'
    )
    command = ["cargo", "metadata", "--manifest-path", str(project / "Cargo.toml"),
               "--format-version", "1"]
    metadata = json.loads(subprocess.check_output(command))
    subprocess.run([sys.executable, str(root / "scripts/check_dependency_contract.py"),
                    str(project / "Cargo.lock")], check=True)
    incompatible = []
    for package in metadata["packages"]:
        version = package.get("rust_version")
        if version and tuple(int(v) for v in version.split(".")[:2]) > (1, 88):
            incompatible.append(f'{package["name"]}@{package["version"]}: Rust {version}')
    if incompatible:
        raise SystemExit("\n".join(incompatible))
    print("Fresh consumer graph: no banned dependencies; declared MSRV <= Rust 1.88")
