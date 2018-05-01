import base64
import json
import logging
import os
import sys
import urllib2

here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

if not(wpt_root in sys.path):
    sys.path.append(wpt_root)

from tools.wpt.testfiles import get_git_cmd

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def get_pr(repo, owner, sha):
    url = ("https://api.github.com/search/issues?q=type:pr+is:merged+repo:%s/%s+sha:%s" %
           (repo, owner, sha))
    try:
        resp = urllib2.urlopen(url)
        body = resp.read()
    except Exception as e:
        logger.error(e)
        return None

    if resp.code != 200:
        logger.error("Got HTTP status %s. Response:" % resp.code)
        logger.error(body)
        return None

    try:
        data = json.loads(body)
    except ValueError:
        logger.error("Failed to read response as JSON:")
        logger.error(body)
        return None

    items = data["items"]
    if len(items) == 0:
        logger.error("No PR found for %s" % sha)
        return None
    if len(items) > 1:
        logger.warning("Found multiple PRs for %s" % sha)

    pr = items[0]

    return pr["number"]


def tag(repo, owner, sha, tag):
    data = json.dumps({"ref": "refs/tags/%s" % tag,
                       "sha": sha})
    try:
        url = "https://api.github.com/repos/%s/%s/git/refs" % (repo, owner)
        req = urllib2.Request(url, data=data)

        base64string = base64.b64encode('%s' % (os.environ["GH_TOKEN"]))
        req.add_header("Authorization", "Basic %s" % base64string)

        opener = urllib2.build_opener(urllib2.HTTPSHandler())

        resp = opener.open(req)
    except Exception as e:
        logger.error("Tag creation failed:\n%s" % e)
        return False

    if resp.code != 201:
        logger.error("Got HTTP status %s. Response:" % resp.code)
        logger.error(resp.read())
        return False

    logger.info("Tagged %s as %s" % (sha, tag))
    return True


def main():
    owner, repo = os.environ["TRAVIS_REPO_SLUG"].split("/", 1)
    if os.environ["TRAVIS_PULL_REQUEST"] != "false":
        logger.info("Not tagging for PR")
        return
    if os.environ["TRAVIS_BRANCH"] != "master":
        logger.info("Not tagging for non-master branch")
        return

    git = get_git_cmd(wpt_root)
    head_rev = git("rev-parse", "HEAD")

    pr = get_pr(owner, repo, head_rev)
    if pr is None:
        sys.exit(1)
    tagged = tag(owner, repo, head_rev, "merge_pr_%s" % pr)
    if not tagged:
        sys.exit(1)


if __name__ == "__main__":
    main()
