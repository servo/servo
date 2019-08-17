import json
import logging
import os
import subprocess
import sys

import requests

here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

if not(wpt_root in sys.path):
    sys.path.append(wpt_root)

from tools.wpt.testfiles import get_git_cmd

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class Status(object):
    SUCCESS = 0
    FAIL = 1
    NEUTRAL = 0


def run(cmd, return_stdout=False, **kwargs):
    logger.info(" ".join(cmd))
    if return_stdout:
        f = subprocess.check_output
    else:
        f = subprocess.check_call
    return f(cmd, **kwargs)


def create_manifest(path):
    run(["./wpt", "manifest", "-p", path])


def compress_manifest(path):
    for args in [["gzip", "-k", "-f", "--best"],
                 ["bzip2", "-k", "-f", "--best"],
                 ["zstd", "-k", "-f", "--ultra", "-22"]]:
        run(args + [path])


def request(url, desc, data=None, json_data=None, params=None, headers=None):
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
        logger.info("Loading URL %s" % url)
        if json_data is not None or data is not None:
            method = requests.post
            kwargs["json"] = json_data
            kwargs["data"] = data
        else:
            method = requests.get

        resp = method(url, **kwargs)

    except Exception as e:
        logger.error("%s failed:\n%s" % (desc, e))
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
        logger.error("%s failed: Returned data was not JSON Response:" %
                     (desc, resp.status_code))
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


def tag(owner, repo, sha, tag):
    data = {"ref": "refs/tags/%s" % tag,
            "sha": sha}
    url = "https://api.github.com/repos/%s/%s/git/refs" % (owner, repo)

    resp_data = request(url, "Tag creation", json_data=data)
    if not resp_data:
        return False

    logger.info("Tagged %s as %s" % (sha, tag))
    return True


def create_release(manifest_path, owner, repo, sha, tag, summary, body):
    if body:
        body = "%s\n%s" % (summary, body)
    else:
        body = summary

    create_url = "https://api.github.com/repos/%s/%s/releases" % (owner, repo)
    create_data = {"tag_name": tag,
                   "name": tag,
                   "body": body}
    create_data = request(create_url, "Release creation", json_data=create_data)
    if not create_data:
        return False

    # Upload URL contains '{?name,label}' at the end which we want to remove
    upload_url = create_data["upload_url"].split("{", 1)[0]

    success = True

    upload_exts = [".gz", ".bz2", ".zst"]
    for upload_ext in upload_exts:
        upload_filename = "MANIFEST-%s.json%s" % (sha, upload_ext)
        params = {"name": upload_filename,
                  "label": "MANIFEST.json%s" % upload_ext}

        with open("%s%s" % (manifest_path, upload_ext), "rb") as f:
            upload_data = f.read()

        logger.info("Uploading %s bytes" % len(upload_data))

        upload_resp = request(upload_url, "Manifest upload", data=upload_data, params=params,
                              headers={'Content-Type': 'application/octet-stream'})
        if not upload_resp:
            success = False

    return success


def should_run_action():
    with open(os.environ["GITHUB_EVENT_PATH"]) as f:
        event = json.load(f)
        logger.info(json.dumps(event, indent=2))

    if "pull_request" in event:
        logger.info("Not tagging for PR")
        return False
    if event.get("ref") != "refs/heads/master":
        logger.info("Not tagging for ref %s" % event.get("ref"))
        return False
    return True


def main():
    repo_key = "GITHUB_REPOSITORY"

    if not should_run_action():
        return Status.NEUTRAL

    owner, repo = os.environ[repo_key].split("/", 1)

    git = get_git_cmd(wpt_root)
    head_rev = git("rev-parse", "HEAD")

    pr = get_pr(owner, repo, head_rev)
    if pr is None:
        # This should only really happen during testing
        tag_name = "merge_commit_%s" % head_rev
    else:
        tag_name = "merge_pr_%s" % pr

    manifest_path = os.path.expanduser(os.path.join("~", "meta", "MANIFEST.json"))

    os.makedirs(os.path.dirname(manifest_path))

    create_manifest(manifest_path)

    compress_manifest(manifest_path)

    tagged = tag(owner, repo, head_rev, tag_name)
    if not tagged:
        return Status.FAIL

    summary = git("show", "--no-patch", '--format="%s"', "HEAD")
    body = git("show", "--no-patch", '--format="%b"', "HEAD")

    if not create_release(manifest_path, owner, repo, head_rev, tag_name, summary, body):
        return Status.FAIL

    return Status.SUCCESS


if __name__ == "__main__":
    code = main()
    assert isinstance(code, int)
    sys.exit(code)
