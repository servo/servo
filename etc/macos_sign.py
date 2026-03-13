#!/usr/bin/env python3

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
#
# This script handles codesigning and notarization of macOS Servo artifacts,
# which is necessary for gatekeeper to allow servoshell to run on macOS.
# The script is intended to be run only be the codesigning maintainer, or
# in github CI (not implemented at time of writing).
#
# Codesigning can take quite a bit of time, so we archive the dmg into etc/notarization,
# to avoid accidentally deleting the artifact (e.g. by `cargo clean`).
# Run this script with --help for usage instructions.

# Since this script handles secrets, it should limit itself to the python standard library
# without adding other dependencies. Hence, we also don't reuse code from mach, and instead
# duplicated what is necessary into this script.

import argparse
import codecs
import json
import logging
import os
import re
import select
import shutil
import subprocess
import sys
import tempfile
import time

logger = logging.getLogger(__name__)


def repo_root() -> str:
    return os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir))


def default_entitlements_path() -> str:
    return os.path.join(repo_root(), "support", "macos", "Servo.entitlements")


def notarization_dir() -> str:
    return os.path.join(repo_root(), "etc", "notarization")


def ensure_notarization_dir() -> str:
    path = notarization_dir()
    os.makedirs(path, exist_ok=True)
    return path


def archive_basename(original_path: str) -> str:
    timestamp = time.strftime("%Y%m%d-%H%M%S", time.localtime())
    return f"{timestamp}-{os.path.basename(original_path)}"


def codesign_app(app_path: str, identity: str, entitlements: str | None) -> None:
    args = ["codesign", "--force", "--deep", "--options", "runtime", "--timestamp", "--sign", identity]
    if entitlements:
        args += ["--entitlements", entitlements]
    args.append(app_path)
    subprocess.check_call(args)
    verify_app_bundle(app_path)


def verify_app_bundle(app_path: str) -> None:
    subprocess.check_call(["codesign", "--verify", "--deep", "--strict", "--verbose=4", app_path])


def notarize_artifact(
    artifact_path: str,
    apple_id: str | None,
    team_id: str | None,
    password: str | None,
    keychain_profile: str | None,
    request_id_path: str | None,
) -> None:
    """
    Submit a notarization request for the given artifact. We don't wait for the request to complete,
    since we expect this to take long. Instead, we save the request ID for later checking.
    """
    cmd = ["xcrun", "notarytool", "submit", artifact_path, "--no-wait", "--progress"]
    if keychain_profile:
        cmd += ["--keychain-profile", keychain_profile]
    else:
        if apple_id is None or team_id is None or password is None:
            raise ValueError(
                "`notarize_artifact` must be called with either \
            keychain_profile OR (apple_id AND team_id AND password) set."
            )
        cmd += ["--apple-id", apple_id, "--team-id", team_id, "--password", password]
    # Uploading can take a while depending on the internet connection, so we use the wrapper to be
    # be able to see the output live while also capturing the output. Kind of like ` | tee` in bash.
    result = run_with_live_output(cmd)
    request_id = extract_request_id(result)
    if request_id and request_id_path:
        with open(request_id_path, "w", encoding="utf-8") as request_file:
            request_file.write(request_id + "\n")


def extract_request_id(output: str) -> str | None:
    match = re.search(r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b", output)
    if match:
        return match.group(0)
    logger.error("Failed to parse notarization request output: %s", output)
    return None


def notarytool_info(
    request_id: str,
    apple_id: str | None,
    team_id: str | None,
    password: str | None,
    keychain_profile: str | None,
) -> dict:
    args = ["xcrun", "notarytool", "info", request_id, "--output-format", "json"]
    if keychain_profile:
        args += ["--keychain-profile", keychain_profile]
    else:
        args += ["--apple-id", apple_id, "--team-id", team_id, "--password", password]
    logger.debug("Running: notarytool info %s", request_id)
    result = subprocess.check_output(args, text=True)
    return json.loads(result)


def read_request_id(path: str) -> str:
    try:
        with open(path, "r", encoding="utf-8") as request_file:
            value = request_file.read().strip()
        if value == "":
            raise ValueError(f"Request ID file is empty (`{path}`).")
        return value
    except OSError:
        raise ValueError(f"Request ID file does not exist: {path}")


def run_with_live_output(cmd: list[str]) -> str:
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        bufsize=0,
    )
    """
    This wrapper allows us to run the command while capturing the output (so we can analyze it later)
    and still printing the output live to the terminal (so we can see progress).
    Note: There is a package (subprocess-tee), which does exactly this (and probably better), but we don't want
    external dependencies, hence implemented ourselves.
    """
    output_chunks: list[str] = []
    if process.stdout is None:
        raise RuntimeError("Failed to capture output.")
    fd = process.stdout.fileno()
    os.set_blocking(fd, False)
    decoder = codecs.getincrementaldecoder("utf-8")("replace")
    while True:
        ready_fds, _, _ = select.select([fd], [], [], 0.2)
        if ready_fds:
            try:
                chunk = os.read(fd, 4096)
            except BlockingIOError:
                chunk = b""
            if chunk:
                text = decoder.decode(chunk)
                output_chunks.append(text)
                print(text, end="", flush=True)
                continue
        if process.poll() is not None:
            while True:
                try:
                    chunk = os.read(fd, 4096)
                except BlockingIOError:
                    chunk = b""
                if not chunk:
                    break
                text = decoder.decode(chunk)
                output_chunks.append(text)
                print(text, end="", flush=True)
            break
        time.sleep(0.05)
    output = "".join(output_chunks)
    if process.returncode != 0:
        raise subprocess.CalledProcessError(process.returncode, cmd, output=output)
    return output


def archive_dmg(dmg_path: str) -> str:
    archive_dir = ensure_notarization_dir()
    archived_name = archive_basename(dmg_path)
    archived_path = os.path.join(archive_dir, archived_name)
    shutil.copyfile(dmg_path, archived_path)
    return archived_path


def latest_archived_dmg() -> str | None:
    directory = notarization_dir()
    try:
        candidates = [os.path.join(directory, name) for name in os.listdir(directory) if name.endswith(".dmg")]
    except OSError:
        return None
    if not candidates:
        return None
    candidates.sort(key=lambda p: os.path.getmtime(p), reverse=True)
    return candidates[0]


def staple_and_verify(dmg_path: str) -> None:
    subprocess.check_call(["xcrun", "stapler", "staple", dmg_path])


def create_dmg(app_path: str, dmg_path: str, identity: str, entitlements: str | None) -> None:
    app_name = os.path.basename(app_path)
    if not app_name.endswith(".app"):
        raise ValueError(f"Expected an .app bundle, got {app_path}")
    if os.path.exists(dmg_path):
        os.remove(dmg_path)
    with tempfile.TemporaryDirectory() as staging_dir:
        staged_app = os.path.join(staging_dir, app_name)
        shutil.copytree(app_path, staged_app)
        # Note: For some reason we need to sign in the dmg staging folder.
        # If we sign before copying the app to the folder, the signature seems to get lost.
        codesign_app(staged_app, identity, entitlements)
        os.symlink("/Applications", os.path.join(staging_dir, "Applications"))
        # Note: `./mach package` retries this command several times, since it can fail in
        # github actions sometimes. If we add github actions support, we may need to do something similar.
        subprocess.check_call(
            [
                "hdiutil",
                "create",
                "-volname",
                "Servo",
                "-megabytes",
                "900",
                dmg_path,
                "-srcfolder",
                staging_dir,
            ]
        )


def verify_app_in_dmg(dmg_path: str) -> None:
    """
    Verify the `.app` bundle in the dmg is still code-signed.
    This is mostly a regression protection, since apparently the .app bundle may not be moved
    after signing.This moves the failure to before submitting the notarization, in case we
    refactor the script and regress.
    """
    with tempfile.TemporaryDirectory() as mount_dir:
        subprocess.check_call(["hdiutil", "attach", "-nobrowse", "-readonly", "-mountpoint", mount_dir, dmg_path])
        try:
            app_candidates = [name for name in os.listdir(mount_dir) if name.endswith(".app")]
            if not app_candidates:
                raise ValueError(f"No .app bundle found in dmg: {dmg_path}")
            if len(app_candidates) > 1:
                raise ValueError(f"Multiple .app bundles found in dmg: {dmg_path}")
            app_path = os.path.join(mount_dir, app_candidates[0])
            verify_app_bundle(app_path)
        finally:
            subprocess.check_call(["hdiutil", "detach", mount_dir])


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Codesign and/or notarize macOS Servo artifacts.\n\n"
            "Preparation (in no particular order):\n"
            "  - Build and package Servo via `./mach build --production && ./mach package --production --preserve-app`\n"
            "  - Setup the required secrets for notarization in keychain:\n"
            "  - Set `SERVO_CODESIGN_IDENTITY` and `SERVO_NOTARY_KEYCHAIN_PROFILE` environment variables.\n\n"
            "Example script usage:\n"
            "  1) Sign the app inside and request notarization: \n"
            "     `./etc/macos_sign.py --app target/production/Servo.app --sign --notarize`\n"
            "  2) Check the status of the (latest) notarization request: \n"
            "     `./etc/macos_sign.py --check-status`\n"
            "  3) Once notarization has passed (this may take a day), staple the dmg and verify it: \n"
            "     `./etc/macos_sign.py --check-status --staple-if-accepted`\n"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--dmg",
        help="Path to dmg to create from the signed app and notarize/staple.",
    )
    parser.add_argument(
        "--app",
        help="Path to a .app bundle to sign.",
    )
    parser.add_argument(
        "--codesign-identity",
        default=os.environ.get("SERVO_CODESIGN_IDENTITY"),
        help="Codesign identity (or set SERVO_CODESIGN_IDENTITY).",
    )
    parser.add_argument(
        "--sign",
        action="store_true",
        help="Codesign the app and package it into a dmg.",
    )
    parser.add_argument(
        "--codesign-entitlements",
        default=os.environ.get("SERVO_CODESIGN_ENTITLEMENTS"),
        help="Entitlements plist path (or set SERVO_CODESIGN_ENTITLEMENTS).",
    )
    parser.add_argument(
        "--notarize",
        action="store_true",
        help="Notarize the dmg (requires notary credentials). This archives the dmg to ./etc/notarization",
    )
    parser.add_argument(
        "--check-status",
        action="store_true",
        help="Check notarization status and optionally staple on success.",
    )
    parser.add_argument(
        "--staple-if-accepted",
        action="store_true",
        help="When checking status, staple and verify the dmg if accepted.",
    )
    parser.add_argument(
        "--notary-apple-id",
        default=os.environ.get("SERVO_NOTARY_APPLE_ID"),
        help="Apple ID for notarization (or set SERVO_NOTARY_APPLE_ID).",
    )
    parser.add_argument(
        "--notary-team-id",
        default=os.environ.get("SERVO_NOTARY_TEAM_ID"),
        help="Apple Team ID for notarization (or set SERVO_NOTARY_TEAM_ID).",
    )
    parser.add_argument(
        "--notary-password",
        default=os.environ.get("SERVO_NOTARY_PASSWORD"),
        help="App-specific password for notarization (or set SERVO_NOTARY_PASSWORD).",
    )
    parser.add_argument(
        "--notary-keychain-profile",
        default=os.environ.get("SERVO_NOTARY_KEYCHAIN_PROFILE"),
        help="Keychain profile for notarytool (or set SERVO_NOTARY_KEYCHAIN_PROFILE).",
    )
    args = parser.parse_args()
    if args.dmg:
        if not args.dmg.endswith(".dmg"):
            parser.error("--dmg must point to a .dmg file.")
        if not os.path.isfile(args.dmg):
            parser.error(f"--dmg does not exist: {args.dmg}")
    return args


def main() -> int:
    logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")
    args = parse_args()

    ensure_notarization_dir()
    dmg_path = args.dmg

    # We don't strictly need the ability to customize / have multiple possible entitlements yet,
    # but hopefully this makes the script easier to reuse for an embedder / more future-proof.
    entitlements = args.codesign_entitlements
    if not entitlements:
        entitlements = default_entitlements_path()

    if args.sign:
        if not args.codesign_identity:
            logger.error("--sign requires --codesign-identity or SERVO_CODESIGN_IDENTITY.")
            return 2
        if not args.app:
            logger.error("--sign requires --app.")
            return 2
        app_path = args.app
        if not app_path.endswith(".app"):
            logger.error("--app must point to a .app bundle, got %s", app_path)
            return 2
        if not os.path.exists(app_path):
            logger.error("--app does not exist: %s", app_path)
            return 2
        with tempfile.TemporaryDirectory() as temp_dir:
            dmg_to_create = os.path.join(temp_dir, "servo-tech-demo.dmg")
            logger.info("Signing app in staging and packaging dmg at %s", dmg_to_create)
            try:
                create_dmg(app_path, dmg_to_create, args.codesign_identity, entitlements)
            except (subprocess.CalledProcessError, ValueError) as e:
                logger.error("DMG creation or code signing failed: %s", e)
                return 1
            archived_name = archive_basename(dmg_to_create)
            dmg_path = os.path.join(ensure_notarization_dir(), archived_name)
            shutil.move(dmg_to_create, dmg_path)
        logger.info("Archived dmg to %s", dmg_path)

    if args.notarize:
        if not dmg_path:
            logger.error("--notarize requires `--dmg` OR `--sign --app <app>`.")
            return 2
        if not (
            args.notary_keychain_profile or (args.notary_apple_id and args.notary_team_id and args.notary_password)
        ):
            logger.error("Notarization requires a keychain profile or Apple ID, team ID, and password.")
            return 2
        if dmg_path == args.dmg:
            try:
                dmg_path = archive_dmg(dmg_path)
                logger.info("Archived dmg to %s", dmg_path)
            except OSError as e:
                logger.error("Failed to archive dmg: %s", e)
                return 1
        request_id_path = os.path.join(
            ensure_notarization_dir(),
            os.path.basename(dmg_path) + ".notary-request-id",
        )
        logger.info("Notarizing %s", dmg_path)
        try:
            verify_app_in_dmg(dmg_path)
            notarize_artifact(
                dmg_path,
                args.notary_apple_id,
                args.notary_team_id,
                args.notary_password,
                args.notary_keychain_profile,
                request_id_path,
            )
            logger.info("Wrote notarization request ID to %s", request_id_path)
        except subprocess.CalledProcessError as e:
            logger.error("Notarization failed: %s", e)
            return 1

    if args.check_status:
        if args.dmg:
            notarized_dmg = args.dmg
        else:
            logger.info("No `--dmg` path provided, checking latest archived dmg.")
            notarized_dmg = latest_archived_dmg()
            if not notarized_dmg:
                logger.error("No archived dmg found to infer request ID.")
                return 2
        request_id_file = notarized_dmg + ".notary-request-id"
        logger.info("Reading notarization request ID from %s", request_id_file)
        try:
            request_id = read_request_id(request_id_file)
        except ValueError as e:
            logger.error("Failed to read notarization request ID: %s", e)
            return 1
        if not (
            args.notary_keychain_profile or (args.notary_apple_id and args.notary_team_id and args.notary_password)
        ):
            logger.error("Checking status requires a keychain profile or Apple ID, team ID, and password.")
            return 2
        try:
            info = notarytool_info(
                request_id,
                args.notary_apple_id,
                args.notary_team_id,
                args.notary_password,
                args.notary_keychain_profile,
            )
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            logger.error("Failed to check notarization status: %s", e)
            return 1
        status = info.get("status")
        logger.info("Notarization status: %s", status)
        if status == "Accepted" and args.staple_if_accepted:
            dmg_to_staple = notarized_dmg
            if not dmg_to_staple:
                logger.error("Unable to locate dmg to staple.")
                return 2
            try:
                staple_and_verify(dmg_to_staple)
            except subprocess.CalledProcessError as e:
                logger.error("Stapling failed: %s", e)
                return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
