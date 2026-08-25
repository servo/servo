# mypy: allow-untyped-defs

import json
import logging
import os
import subprocess
import sys
import tempfile

import requests

from pathlib import Path


here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

if wpt_root not in sys.path:
    sys.path.append(wpt_root)

from tools.wpt.testfiles import get_git_cmd

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class Status:
    SUCCESS = 0
    FAIL = 1


def run(cmd, return_stdout=False, **kwargs):
    logger.info(" ".join(cmd))
    if return_stdout:
        f = subprocess.check_output
    else:
        f = subprocess.check_call
    return f(cmd, **kwargs)


def create_manifest(path):
    run(["./wpt", "manifest", "-p", path])


def create_web_feature_manifest(path):
    run(["./wpt", "web-features-manifest", "-p", path])


def compress_manifest(path):
    for args in [["gzip", "-k", "-f", "--best"],
                 ["bzip2", "-k", "-f", "--best"],
                 ["zstd", "-k", "-f", "--ultra", "-22", "-q"]]:
        run(args + [path])


def request(url, desc, method=None, data=None, json_data=None, params=None, headers=None):
    github_token = os.environ.get("GITHUB_TOKEN")
    default_headers = {
        "Authorization": "token %s" % github_token,
        "Accept": "application/vnd.github.machine-man-preview+json"
    }

    _headers = default_headers
    if headers is not None:
        _headers.update(headers)

    kwargs = {"params": params,
              "headers": _headers}
    try:
        logger.info("Requesting URL %s" % url)
        if json_data is not None or data is not None:
            if method is None:
                method = requests.post
            kwargs["json"] = json_data
            kwargs["data"] = data
        elif method is None:
            method = requests.get

        resp = method(url, **kwargs)

    except Exception as e:
        logger.error(f"{desc} failed:\n{e}")
        return None

    try:
        resp.raise_for_status()
    except requests.HTTPError:
        logger.error("%s failed: Got HTTP status %s. Response:" %
                     (desc, resp.status_code))
        logger.error(resp.text)
        return None

    try:
        return resp.json()
    except ValueError:
        logger.error("%s failed: Returned data was not JSON Response:" % desc)
        logger.error(resp.text)


def get_pr(owner, repo, sha):
    data = request("https://api.github.com/search/issues?q=type:pr+is:merged+repo:%s/%s+sha:%s" %
                   (owner, repo, sha), "Getting PR")
    if data is None:
        return None

    items = data["items"]
    if len(items) == 0:
        logger.error("No PR found for %s" % sha)
        return None
    if len(items) > 1:
        logger.warning("Found multiple PRs for %s" % sha)

    pr = items[0]

    return pr["number"]


def get_file_upload_details(manifest_path, sha):
    """
    For a given file, generate details used to upload to GitHub.
    """
    path = Path(manifest_path)
    stem = path.stem
    extension = path.suffix
    upload_filename_prefix = f"{stem}-{sha}{extension}"
    upload_label_prefix = path.name
    upload_desc = f"{stem.title()} upload"
    return upload_filename_prefix, upload_label_prefix, upload_desc


def create_release(manifest_file_paths, owner, repo, sha, tag, body):
    logger.info(f"Creating a release for tag='{tag}', target_commitish='{sha}'")
    create_url = f"https://api.github.com/repos/{owner}/{repo}/releases"
    create_data = {"tag_name": tag,
                   "target_commitish": sha,
                   "name": tag,
                   "body": body,
                   "draft": True}
    create_resp = request(create_url, "Release creation", json_data=create_data)
    if not create_resp:
        return False

    # Upload URL contains '{?name,label}' at the end which we want to remove
    upload_url = create_resp["upload_url"].split("{", 1)[0]

    upload_exts = [".gz", ".bz2", ".zst"]
    for manifest_path in manifest_file_paths:
        upload_filename_prefix, upload_label_prefix, upload_desc = get_file_upload_details(manifest_path, sha)
        for upload_ext in upload_exts:
            upload_filename = f"{upload_filename_prefix}{upload_ext}"
            params = {"name": upload_filename,
                    "label": f"{upload_label_prefix}{upload_ext}"}

            with open(f"{manifest_path}{upload_ext}", "rb") as f:
                upload_data = f.read()

            logger.info("Uploading %s bytes" % len(upload_data))

            upload_resp = request(upload_url, upload_desc, data=upload_data, params=params,
                                headers={'Content-Type': 'application/octet-stream'})
            if not upload_resp:
                return False

    release_id = create_resp["id"]
    edit_url = f"https://api.github.com/repos/{owner}/{repo}/releases/{release_id}"
    edit_data = create_data.copy()
    edit_data["draft"] = False
    edit_resp = request(edit_url, "Release publishing", method=requests.patch, json_data=edit_data)
    if not edit_resp:
        return False

    logger.info("Released %s" % edit_resp["html_url"])
    return True


def should_dry_run():
    with open(os.environ["GITHUB_EVENT_PATH"]) as f:
        event = json.load(f)
        logger.info(json.dumps(event, indent=2))

    if "pull_request" in event:
        logger.info("Dry run for PR")
        return True
    if event.get("ref") != "refs/heads/master":
        logger.info("Dry run for ref %s" % event.get("ref"))
        return True
    return False


def main():
    dry_run = should_dry_run()

    manifest_path = os.path.join(tempfile.mkdtemp(), "MANIFEST.json")
    web_features_manifest_path = os.path.join(tempfile.mkdtemp(), "WEB_FEATURES_MANIFEST.json")

    create_manifest(manifest_path)
    create_web_feature_manifest(web_features_manifest_path)

    compress_manifest(manifest_path)
    compress_manifest(web_features_manifest_path)

    owner, repo = os.environ["GITHUB_REPOSITORY"].split("/", 1)

    git = get_git_cmd(wpt_root)
    head_rev = git("rev-parse", "HEAD").strip()
    body = git("show", "--no-patch", "--format=%B", "HEAD")

    if dry_run:
        return Status.SUCCESS

    pr = get_pr(owner, repo, head_rev)
    if pr is None:
        return Status.FAIL
    tag_name = "merge_pr_%s" % pr

    manifest_paths = [manifest_path, web_features_manifest_path]
    if not create_release(manifest_paths, owner, repo, head_rev, tag_name, body):
        return Status.FAIL

    return Status.SUCCESS


if __name__ == "__main__":
    code = main()  # type: ignore
    assert isinstance(code, int)
    sys.exit(code)
