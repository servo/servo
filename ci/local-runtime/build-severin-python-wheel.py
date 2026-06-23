#!/usr/bin/env python3
"""Assemble the experimental Severin CPython extension wheel.

This intentionally avoids a packaging framework: Cargo builds the native module,
and this script stages the Cargo cdylib under the exact extension filename that
this CPython interpreter imports, then writes the minimal wheel metadata needed
for offline `pip install --no-deps`.
"""

from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import os
import re
import shutil
import subprocess
import sys
import sysconfig
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


def workspace_version(repo: Path) -> str:
    cargo_toml = repo / "Cargo.toml"
    text = cargo_toml.read_text(encoding="utf-8")
    in_workspace_package = False
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if line.startswith("[") and line.endswith("]"):
            in_workspace_package = line == "[workspace.package]"
            continue
        if in_workspace_package:
            match = re.match(r'version\s*=\s*"([^"]+)"', line)
            if match:
                return match.group(1)
    raise SystemExit("Could not find [workspace.package] version in Cargo.toml")


def python_tags() -> tuple[str, str, str, str]:
    ext_suffix = sysconfig.get_config_var("EXT_SUFFIX")
    if not ext_suffix:
        raise SystemExit("CPython did not report EXT_SUFFIX")

    soabi = sysconfig.get_config_var("SOABI") or ""
    version = sys.version_info
    python_tag = f"cp{version.major}{version.minor}"

    if soabi.startswith(f"cpython-{version.major}{version.minor}"):
        abi_tag = python_tag
    else:
        raise SystemExit(f"Unsupported or non-CPython SOABI for this wheel: {soabi!r}")

    platform = sysconfig.get_platform().replace("-", "_").replace(".", "_")
    return ext_suffix, python_tag, abi_tag, platform


def wheel_hash(data: bytes) -> str:
    digest = hashlib.sha256(data).digest()
    return "sha256=" + base64.urlsafe_b64encode(digest).rstrip(b"=").decode("ascii")


def write_wheel_record(wheel_path: Path) -> None:
    rows: list[list[str]] = []
    record_name = next(name for name in ZipFile(wheel_path).namelist() if name.endswith(".dist-info/RECORD"))
    with ZipFile(wheel_path, "r") as zf:
        for name in zf.namelist():
            if name == record_name:
                rows.append([name, "", ""])
            else:
                data = zf.read(name)
                rows.append([name, wheel_hash(data), str(len(data))])

    rendered = []
    for row in rows:
        from io import StringIO

        buf = StringIO()
        csv.writer(buf, lineterminator="\n").writerow(row)
        rendered.append(buf.getvalue())
    record_text = "".join(rendered)

    tmp_path = wheel_path.with_suffix(".tmp.whl")
    with ZipFile(wheel_path, "r") as zin, ZipFile(
        tmp_path, "w", compression=ZIP_DEFLATED, compresslevel=9, strict_timestamps=False
    ) as zout:
        for item in zin.infolist():
            if item.filename == record_name:
                continue
            zout.writestr(item, zin.read(item.filename))
        zout.writestr(record_name, record_text)
    tmp_path.replace(wheel_path)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=os.environ.get("GITHUB_WORKSPACE", "."))
    parser.add_argument("--target-dir", default=os.environ.get("CARGO_TARGET_DIR", "target"))
    parser.add_argument("--profile", default="release")
    parser.add_argument("--output-dir", default="release")
    parser.add_argument("--skip-cargo-build", action="store_true")
    args = parser.parse_args()

    repo = Path(args.repo).resolve()
    target_dir = Path(args.target_dir).resolve()
    output_dir = (repo / args.output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    if not args.skip_cargo_build:
        subprocess.run(
            ["cargo", "build", "-p", "severin-python", "--release"],
            cwd=repo,
            check=True,
        )

    version = workspace_version(repo)
    ext_suffix, python_tag, abi_tag, platform_tag = python_tags()
    wheel_name = f"severin-{version}-{python_tag}-{abi_tag}-{platform_tag}.whl"
    dist_info = f"severin-{version}.dist-info"
    wheel_path = output_dir / wheel_name

    cargo_lib = target_dir / args.profile / "libseverin.so"
    if not cargo_lib.exists():
        raise SystemExit(f"Cargo output not found: {cargo_lib}")

    extension_name = f"severin{ext_suffix}"
    staged_extension = output_dir / extension_name
    shutil.copy2(cargo_lib, staged_extension)

    metadata = (
        "Metadata-Version: 2.1\n"
        "Name: severin\n"
        f"Version: {version}\n"
        "Summary: Experimental in-process CPython bindings for the Severin local Servo runtime.\n"
        "License: MPL-2.0\n"
        "Requires-Python: >=3.11\n"
    )
    wheel = (
        "Wheel-Version: 1.0\n"
        "Generator: ci/local-runtime/build-severin-python-wheel.py\n"
        "Root-Is-Purelib: false\n"
        f"Tag: {python_tag}-{abi_tag}-{platform_tag}\n"
    )

    with ZipFile(
        wheel_path, "w", compression=ZIP_DEFLATED, compresslevel=9, strict_timestamps=False
    ) as zf:
        zf.write(staged_extension, extension_name)
        zf.writestr(f"{dist_info}/METADATA", metadata)
        zf.writestr(f"{dist_info}/WHEEL", wheel)
        zf.writestr(f"{dist_info}/RECORD", "")
    staged_extension.unlink()
    write_wheel_record(wheel_path)

    print(f"wheel={wheel_path}")
    print(f"wheel_name={wheel_name}")
    print(f"python_tag={python_tag}")
    print(f"abi_tag={abi_tag}")
    print(f"platform_tag={platform_tag}")
    print(f"extension_suffix={ext_suffix}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
