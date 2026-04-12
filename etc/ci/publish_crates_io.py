#!/usr/bin/env python3

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable
from urllib import error, parse, request

# Allow crates.io team to easily identify us incase this script misbehaves.
USER_AGENT = "servo-publish-crates-io/1 (https://github.com/servo/servo)"

WORKSPACE_ROOT = Path(__file__).resolve().parents[2]
SLEEP_AFTER_PUBLISH_SECONDS = int(os.environ.get("SERVO_CRATES_IO_SLEEP_AFTER_PUBLISH_SECONDS", "30"))
VERIFY_PUBLISHED_TIMEOUT_SECONDS = int(os.environ.get("SERVO_CRATES_IO_VERIFY_PUBLISHED_TIMEOUT_SECONDS", "300"))
VERIFY_PUBLISHED_INTERVAL_SECONDS = int(os.environ.get("SERVO_CRATES_IO_VERIFY_PUBLISHED_INTERVAL_SECONDS", "10"))
API_TIMEOUT_SECONDS = int(os.environ.get("SERVO_CRATES_IO_API_TIMEOUT_SECONDS", "30"))


@dataclass(frozen=True)
class WorkspacePackage:
    name: str
    version: str
    manifest_path: str
    dependencies: tuple[str, ...]


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Publish workspace crates to crates.io in dependency order, skipping "
            "versions that already exist and waiting for index propagation after each publish."
        )
    )
    parser.add_argument(
        "--no-verify",
        action="store_true",
        help="Pass --no-verify to cargo publish.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the resolved publish order without querying crates.io or publishing.",
    )
    return parser.parse_args()


def load_metadata() -> dict:
    return json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=WORKSPACE_ROOT,
            text=True,
        )
    )


def publishes_to_crates_io(package: dict) -> bool:
    publish = package.get("publish")
    # See <https://doc.rust-lang.org/cargo/commands/cargo-metadata.html>
    # > List of registries to which this package may be published.
    # > Publishing is unrestricted if null, and forbidden if an empty array.
    return publish is None or "crates-io" in publish


def collect_packages(metadata: dict) -> dict[str, WorkspacePackage]:
    local_packages = {package["name"]: package for package in metadata["packages"] if package.get("source") is None}
    publishable_names = {name for name, package in local_packages.items() if publishes_to_crates_io(package)}

    packages: dict[str, WorkspacePackage] = {}
    for name in sorted(publishable_names):
        package = local_packages[name]
        dependencies = set()
        blocked_dependencies = set()
        for dependency in package["dependencies"]:
            dependency_name = dependency["name"]
            if dependency.get("kind") == "dev":
                continue
            if dependency.get("path") is None or dependency_name not in local_packages:
                continue
            if dependency_name in publishable_names:
                dependencies.add(dependency_name)
            else:
                blocked_dependencies.add(dependency_name)

        if blocked_dependencies:
            blocked = ", ".join(sorted(blocked_dependencies))
            raise ValueError(f"{name} depends on local crate(s) that do not publish to crates.io: {blocked}")

        packages[name] = WorkspacePackage(
            name=name,
            version=package["version"],
            manifest_path=package["manifest_path"],
            dependencies=tuple(sorted(dependencies)),
        )

    return packages


def topological_publish_order(packages: dict[str, WorkspacePackage]) -> list[WorkspacePackage]:
    """
    If there were a command like `cargo publish --workspace --print`, which just prints a
    publishable order of crates, then we would use that. Since that doesn't exist we
    implement this ourselves by walking the dependency tree.
    In principle we have a list of WorkspacePackage objects, where each entry has a list of
    in-workspace dependencies. We start from the edges (no dependencies), and work our way
    to the top, in each iteration appending to the publish-list, and removing the entries
    from the dependency lists of the remaining crates.
    """
    remaining = {name: set(package.dependencies) for name, package in packages.items()}
    ordered_names: list[str] = []

    while remaining:
        ready = sorted(name for name, dependencies in remaining.items() if not dependencies)
        assert ready is not None and len(ready) > 0, "Unable to resolve publish order"

        ordered_names.extend(ready)
        for name in ready:
            remaining.pop(name)
        for dependencies in remaining.values():
            dependencies.difference_update(ready)

    return [packages[name] for name in ordered_names]


def crates_io_version_exists(crate_name: str, version: str) -> bool:
    crate_path = parse.quote(crate_name, safe="")
    version_path = parse.quote(version, safe="")
    api_url = f"https://crates.io/api/v1/crates/{crate_path}/{version_path}"
    req = request.Request(api_url, headers={"User-Agent": USER_AGENT})
    try:
        with request.urlopen(req, timeout=API_TIMEOUT_SECONDS):
            return True
    except error.HTTPError as http_error:
        if http_error.code == 404:
            return False
        raise RuntimeError(
            f"crates.io returned HTTP {http_error.code} while checking {crate_name} {version}"
        ) from http_error
    except error.URLError as url_error:
        raise RuntimeError(f"failed to query crates.io for {crate_name} {version}: {url_error.reason}") from url_error


def wait_until_published(package: WorkspacePackage) -> None:
    deadline = time.monotonic() + VERIFY_PUBLISHED_TIMEOUT_SECONDS
    while True:
        try:
            if crates_io_version_exists(package.name, package.version):
                log(f"verified {package.name} {package.version} on crates.io")
                return
        except RuntimeError as runtime_error:
            log(str(runtime_error))

        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise RuntimeError(f"timed out waiting for {package.name} {package.version} to appear on crates.io")

        time.sleep(min(float(VERIFY_PUBLISHED_INTERVAL_SECONDS), remaining))


def publish_package(
    args: argparse.Namespace,
    package: WorkspacePackage,
) -> None:
    command = ["cargo", "publish", "--manifest-path", package.manifest_path]
    if args.no_verify:
        command.append("--no-verify")

    log(f"publishing {package.name} {package.version}")
    subprocess.run(command, cwd=WORKSPACE_ROOT, check=True)


def publish_packages(args: argparse.Namespace, packages: Iterable[WorkspacePackage]) -> None:
    for package in packages:
        if crates_io_version_exists(package.name, package.version):
            log(f"skipping {package.name} {package.version}; already on crates.io")
            continue

        publish_package(args, package)
        duration_seconds = SLEEP_AFTER_PUBLISH_SECONDS
        log(f"published {package.name} {package.version}. Waiting for {duration_seconds}s")
        # To distribute load on crates.io, we sleep for a bit after each publish.
        time.sleep(duration_seconds)
        # And in case crates.io is under heavy load and publishing takes longer than usual,
        # try and wait until the new version appears on crates.io.
        wait_until_published(package)


def main() -> int:
    args = parse_args()

    metadata = load_metadata()
    packages = collect_packages(metadata)
    ordered_packages = topological_publish_order(packages)

    if not ordered_packages:
        log("no local crates to publish to crates.io")
        return 0

    log("publish order:")
    for package in ordered_packages:
        dependency_list = ", ".join(package.dependencies) if package.dependencies else "none"
        log(f"  {package.name} {package.version} (deps: {dependency_list})")

    if args.dry_run:
        return 0

    publish_packages(args, ordered_packages)
    return 0


if __name__ == "__main__":
    sys.exit(main())
