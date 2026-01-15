#!/usr/bin/env -S uv run --script

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
# Note: We use `uv lock --script upload_nightly.py` to lock dependency versions in a lockfile.
# /// script
# requires-python = ">=3.11"
# dependencies = ["boto3", "PyGithub"]
# ///
import argparse
import hashlib
import io
import json
import os
from os import path
from datetime import datetime
from typing import List, Optional

import sys
from github import Github

import boto3


def get_s3_secret(secret_from_environment: bool) -> tuple:
    aws_access_key = None
    aws_secret_access_key = None
    if secret_from_environment:
        secret = json.loads(os.environ["S3_UPLOAD_CREDENTIALS"])
        aws_access_key = secret["aws_access_key_id"]
        aws_secret_access_key = secret["aws_secret_access_key"]
    return (aws_access_key, aws_secret_access_key)


def nightly_filename(package: str, timestamp: datetime) -> str:
    return "{}-{}".format(
        timestamp.isoformat() + "Z",  # The `Z` denotes UTC
        path.basename(package),
    )


# Map the default platform shorthand to a name containing architecture.
def map_platform(platform: str) -> str:
    if platform == "android":
        return "aarch64-android"
    elif platform == "linux":
        return "x86_64-linux-gnu"
    elif platform == "windows-msvc":
        return "x86_64-windows-msvc"
    elif platform == "mac":
        return "x86_64-apple-darwin"
    elif platform == "mac-arm64":
        return "aarch64-apple-darwin"
    elif platform == "ohos":
        return "aarch64-linux-ohos"
    raise Exception("Unknown platform: {}".format(platform))


def upload_to_github_release(platform: str, package: str, package_hash: str, github_release_id: Optional[int]) -> None:
    if not github_release_id:
        return

    extension = path.basename(package).partition(".")[2]
    g = Github(os.environ["NIGHTLY_REPO_TOKEN"])
    nightly_repo = g.get_repo(os.environ["NIGHTLY_REPO"])
    release = nightly_repo.get_release(github_release_id)

    if platform != "mac-arm64":
        # Legacy assetname. Will be removed after a period with duplicate assets.
        asset_name = f"servo-latest.{extension}"
        package_hash_fileobj = io.BytesIO(f"{package_hash}  {asset_name}".encode("utf-8"))
        release.upload_asset(package, name=asset_name)
        # pyrefly: ignore[missing-attribute]
        release.upload_asset_from_memory(
            package_hash_fileobj, package_hash_fileobj.getbuffer().nbytes, name=f"{asset_name}.sha256"
        )
    asset_platform = map_platform(platform)
    asset_name = f"servo-{asset_platform}.{extension}"
    package_hash_fileobj = io.BytesIO(f"{package_hash}  {asset_name}".encode("utf-8"))
    release.upload_asset(package, name=asset_name)
    # pyrefly: ignore[missing-attribute]
    release.upload_asset_from_memory(
        package_hash_fileobj, package_hash_fileobj.getbuffer().nbytes, name=f"{asset_name}.sha256"
    )


def upload_to_s3(
    platform: str, package: str, package_hash: str, timestamp: datetime, secret_from_environment: bool
) -> None:
    (aws_access_key, aws_secret_access_key) = get_s3_secret(secret_from_environment)
    s3 = boto3.client("s3", aws_access_key_id=aws_access_key, aws_secret_access_key=aws_secret_access_key)

    cloudfront = boto3.client(
        "cloudfront", aws_access_key_id=aws_access_key, aws_secret_access_key=aws_secret_access_key
    )

    BUCKET = "servo-builds2"
    DISTRIBUTION_ID = "EJ8ZWSJKFCJS2"

    nightly_dir = f"nightly/{platform}"
    filename = nightly_filename(package, timestamp)
    package_upload_key = "{}/{}".format(nightly_dir, filename)
    extension = path.basename(package).partition(".")[2]
    latest_upload_key = "{}/servo-latest.{}".format(nightly_dir, extension)

    package_hash_fileobj = io.BytesIO(f"{package_hash}  {filename}".encode("utf-8"))
    latest_hash_upload_key = f"{latest_upload_key}.sha256"

    s3.upload_file(package, BUCKET, package_upload_key)

    copy_source = {
        "Bucket": BUCKET,
        "Key": package_upload_key,
    }
    s3.copy(copy_source, BUCKET, latest_upload_key)
    s3.upload_fileobj(package_hash_fileobj, BUCKET, latest_hash_upload_key, ExtraArgs={"ContentType": "text/plain"})

    # Invalidate previous "latest" nightly files from
    # CloudFront edge caches
    cloudfront.create_invalidation(
        DistributionId=DISTRIBUTION_ID,
        InvalidationBatch={
            "CallerReference": f"{latest_upload_key}-{timestamp}",
            "Paths": {"Quantity": 1, "Items": [f"/{latest_upload_key}*"]},
        },
    )


def upload_nightly(
    platform: str, secret_from_environment: bool, github_release_id: int | None, packages: List[str]
) -> int:
    timestamp = datetime.utcnow().replace(microsecond=0)
    for package in packages:
        # TODO: This if feels like it should not be necessary. Let's add a warning, and make this an
        # error later.
        if path.isdir(package):
            print("Warning: Skipping directory: {}".format(package), file=sys.stderr)
            continue
        if not path.isfile(package):
            print("Could not find package for {} at {}".format(platform, package), file=sys.stderr)
            return 1

        # Compute the hash
        SHA_BUF_SIZE = 1048576  # read in 1 MiB chunks
        sha256_digest = hashlib.sha256()
        with open(package, "rb") as package_file:
            while True:
                data = package_file.read(SHA_BUF_SIZE)
                if not data:
                    break
                sha256_digest.update(data)
        package_hash = sha256_digest.hexdigest()

        upload_to_s3(platform, package, package_hash, timestamp, secret_from_environment)
        upload_to_github_release(platform, package, package_hash, github_release_id)

    return 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Upload Servo nightly to Github Releases and S3")
    parser.add_argument("platform", help="Package platform type to upload")
    parser.add_argument(
        "--secret-from-environment", action="store_true", help="Retrieve the appropriate secrets from the environment."
    )
    parser.add_argument(
        "--github-release-id", default=None, type=int, help="The github release to upload the nightly builds."
    )
    parser.add_argument("packages", nargs="+", help="The packages to upload.")
    args = parser.parse_args()
    upload_nightly(
        platform=args.platform,
        secret_from_environment=args.secret_from_environment,
        github_release_id=args.github_release_id,
        packages=args.packages,
    )
