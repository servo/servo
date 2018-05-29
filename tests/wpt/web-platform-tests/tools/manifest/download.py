from __future__ import absolute_import

import argparse
import gzip
import json
import io
import os
from datetime import datetime, timedelta

from six.moves.urllib.request import urlopen

from .vcs import Git

from . import log

here = os.path.dirname(__file__)

wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))
logger = log.get_logger()


def abs_path(path):
    return os.path.abspath(os.path.expanduser(path))


def should_download(manifest_path, rebuild_time=timedelta(days=5)):
    if not os.path.exists(manifest_path):
        return True
    mtime = datetime.fromtimestamp(os.path.getmtime(manifest_path))
    if mtime < datetime.now() - rebuild_time:
        return True
    logger.info("Skipping manifest download because existing file is recent")
    return False


def git_commits(repo_root):
    git = Git.get_func(repo_root)
    return [item for item in git("log", "--format=%H", "-n50").split("\n") if item]


def github_url(commits):
    try:
        resp = urlopen("https://api.github.com/repos/web-platform-tests/wpt/releases")
    except Exception:
        return None

    if resp.code != 200:
        return None

    try:
        releases = json.load(resp.fp)
    except ValueError:
        logger.warning("Response was not valid JSON")
        return None

    fallback = None
    for release in releases:
        for commit in commits:
            for item in release["assets"]:
                if item["name"] == "MANIFEST-%s.json.gz" % commit:
                    return item["browser_download_url"]
                elif item["name"] == "MANIFEST.json.gz" and not fallback:
                    fallback = item["browser_download_url"]
    if fallback:
        logger.info("Can't find a commit-specific manifest so just using the most recent one")
        return fallback


def download_manifest(manifest_path, commits_func, url_func, force=False):
    if not force and not should_download(manifest_path):
        return False

    commits = commits_func()

    url = url_func(commits)
    if not url:
        logger.warning("No generated manifest found")
        return False

    logger.info("Downloading manifest from %s" % url)
    try:
        resp = urlopen(url)
    except Exception:
        logger.warning("Downloading pregenerated manifest failed")
        return False

    if resp.code != 200:
        logger.warning("Downloading pregenerated manifest failed; got HTTP status %d" %
                       resp.code)
        return False

    gzf = gzip.GzipFile(fileobj=io.BytesIO(resp.read()))

    try:
        decompressed = gzf.read()
    except IOError:
        logger.warning("Failed to decompress downloaded file")
        return False

    try:
        with open(manifest_path, "w") as f:
            f.write(decompressed)
    except Exception:
        logger.warning("Failed to write manifest")
        return False
    logger.info("Manifest downloaded")
    return True


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-p", "--path", type=abs_path, help="Path to manifest file.")
    parser.add_argument(
        "--tests-root", type=abs_path, default=wpt_root, help="Path to root of tests.")
    parser.add_argument(
        "--force", action="store_true",
        help="Always download, even if the existing manifest is recent")
    return parser


def download_from_github(path, tests_root, force=False):
    return download_manifest(path, lambda: git_commits(tests_root), github_url,
                             force=force)


def run(**kwargs):
    if kwargs["path"] is None:
        path = os.path.join(kwargs["tests_root"], "MANIFEST.json")
    else:
        path = kwargs["path"]
    success = download_from_github(path, kwargs["tests_root"], kwargs["force"])
    return 0 if success else 1
