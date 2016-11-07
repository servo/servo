import argparse
import json
import logging
import os
import re
import stat
import subprocess
import sys
import tarfile
import traceback
import zipfile
from cStringIO import StringIO
from collections import defaultdict
from urlparse import urljoin

import requests

BaseHandler = None
LogActionFilter = None
LogHandler = None
LogLevelFilter = None
StreamHandler = None
TbplFormatter = None
reader = None
wptcommandline = None
wptrunner = None


logger = logging.getLogger(os.path.splitext(__file__)[0])


def do_delayed_imports():
    global BaseHandler
    global LogLevelFilter
    global StreamHandler
    global TbplFormatter
    global reader
    global wptcommandline
    global wptrunner
    from mozlog import reader
    from mozlog.formatters import TbplFormatter
    from mozlog.handlers import BaseHandler, LogLevelFilter, StreamHandler
    from wptrunner import wptcommandline, wptrunner
    setup_log_handler()
    setup_action_filter()


def setup_logging():
    handler = logging.StreamHandler(sys.stdout)
    formatter = logging.Formatter(logging.BASIC_FORMAT, None)
    handler.setFormatter(formatter)
    logger.addHandler(handler)
    logger.setLevel(logging.DEBUG)

setup_logging()


def setup_action_filter():
    global LogActionFilter

    class LogActionFilter(BaseHandler):
        """Handler that filters out messages with action of log and a level
        lower than some specified level.

        :param inner: Handler to use for messages that pass this filter
        :param level: Minimum log level to process
        """
        def __init__(self, inner, actions):
            BaseHandler.__init__(self, inner)
            self.inner = inner
            self.actions = actions

        def __call__(self, item):
            if item["action"] in self.actions:
                return self.inner(item)


class GitHub(object):
    def __init__(self, org, repo, token):
        self.token = token
        self.headers = {"Accept": "application/vnd.github.v3+json"}
        self.auth = (self.token, "x-oauth-basic")
        self.org = org
        self.repo = repo
        self.base_url = "https://api.github.com/repos/%s/%s/" % (org, repo)

    def _headers(self, headers):
        if headers is None:
            headers = {}
        rv = self.headers.copy()
        rv.update(headers)
        return rv

    def post(self, url, data, headers=None):
        logger.debug("POST %s" % url)
        if data is not None:
            data = json.dumps(data)
        resp = requests.post(
            url,
            data=data,
            headers=self._headers(headers),
            auth=self.auth
        )
        resp.raise_for_status()
        return resp

    def get(self, url, headers=None):
        logger.debug("GET %s" % url)
        resp = requests.get(
            url,
            headers=self._headers(headers),
            auth=self.auth
        )
        resp.raise_for_status()
        return resp

    def post_comment(self, issue_number, body):
        url = urljoin(self.base_url, "issues/%s/comments" % issue_number)
        return self.post(url, {"body": body})

    def releases(self):
        url = urljoin(self.base_url, "releases/latest")
        return self.get(url)


class GitHubCommentHandler(logging.Handler):
    def __init__(self, github, pull_number):
        logging.Handler.__init__(self)
        self.github = github
        self.pull_number = pull_number
        self.log_data = []

    def emit(self, record):
        try:
            msg = self.format(record)
            self.log_data.append(msg)
        except Exception:
            self.handleError(record)

    def send(self):
        self.github.post_comment(self.pull_number, "\n".join(self.log_data))
        self.log_data = []


class Browser(object):
    product = None

    def __init__(self, github_token):
        self.github_token = github_token


class Firefox(Browser):
    product = "firefox"

    def install(self):
        call("pip", "install", "-r", "w3c/wptrunner/requirements_firefox.txt")
        resp = get("https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/firefox-52.0a1.en-US.linux-x86_64.tar.bz2")
        untar(resp.raw)

        if not os.path.exists("profiles"):
            os.mkdir("profiles")
        with open(os.path.join("profiles", "prefs_general.js"), "wb") as f:
            resp = get("https://hg.mozilla.org/mozilla-central/raw-file/tip/testing/profiles/prefs_general.js")
            f.write(resp.content)
        call("pip", "install", "-r", os.path.join("w3c", "wptrunner", "requirements_firefox.txt"))

    def _latest_geckodriver_version(self):
        # This is used rather than an API call to avoid rate limits
        tags = call("git", "ls-remote", "--tags", "--refs",
                    "https://github.com/mozilla/geckodriver.git")
        release_re = re.compile(".*refs/tags/v(\d+)\.(\d+)\.(\d+)")
        latest_release = 0
        for item in tags.split("\n"):
            m = release_re.match(item)
            if m:
                version = [int(item) for item in m.groups()]
                if version > latest_release:
                    latest_release = version
        assert latest_release != 0
        return "v%s.%s.%s" % tuple(str(item) for item in latest_release)


    def install_webdriver(self):
        version = self._latest_geckodriver_version()
        logger.debug("Latest geckodriver release %s" % version)
        url = "https://github.com/mozilla/geckodriver/releases/download/%s/geckodriver-%s-linux64.tar.gz" % (version, version)
        untar(get(url).raw)

    def wptrunner_args(self, root):
        return {
            "product": "firefox",
            "binary": "%s/firefox/firefox" % root,
            "certutil_binary": "certutil",
            "webdriver_binary": "%s/geckodriver" % root,
            "prefs_root": "%s/profiles" % root,
        }


class Chrome(Browser):
    product = "chrome"

    def install(self):
        latest = get("https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/Linux_x64%2FLAST_CHANGE?alt=media").text.strip()
        url = "https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/Linux_x64%%2F%s%%2Fchrome-linux.zip?alt=media" % latest
        unzip(get(url).raw)
        logger.debug(call("ls", "-lhrt", "chrome-linux"))
        call("pip", "install", "-r", os.path.join("w3c", "wptrunner", "requirements_chrome.txt"))

    def install_webdriver(self):
        latest = get("http://chromedriver.storage.googleapis.com/LATEST_RELEASE").text.strip()
        url = "http://chromedriver.storage.googleapis.com/%s/chromedriver_linux64.zip" % latest
        unzip(get(url).raw)
        st = os.stat('chromedriver')
        os.chmod('chromedriver', st.st_mode | stat.S_IEXEC)

    def wptrunner_args(self, root):
        return {
            "product": "chrome",
            "binary": "%s/chrome-linux/chrome" % root,
            "webdriver_binary": "%s/chromedriver" % root,
            "test_types": ["testharness", "reftest"]
        }


def get(url):
    logger.debug("GET %s" % url)
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    return resp


def call(*args):
    logger.debug("%s" % " ".join(args))
    return subprocess.check_output(args)


def get_git_cmd(repo_path):
    def git(cmd, *args):
        full_cmd = ["git", cmd] + list(args)
        try:
            return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT)
        except subprocess.CalledProcessError as e:
            logger.error("Git command exited with status %i" % e.returncode)
            logger.error(e.output)
            sys.exit(1)
    return git


def seekable(fileobj):
    try:
        fileobj.seek(fileobj.tell())
    except Exception:
        return StringIO(fileobj.read())
    else:
        return fileobj


def untar(fileobj):
    logger.debug("untar")
    fileobj = seekable(fileobj)
    with tarfile.open(fileobj=fileobj) as tar_data:
        tar_data.extractall()


def unzip(fileobj):
    logger.debug("unzip")
    fileobj = seekable(fileobj)
    with zipfile.ZipFile(fileobj) as zip_data:
        for info in zip_data.infolist():
            zip_data.extract(info)
            perm = info.external_attr >> 16 & 0x1FF
            os.chmod(info.filename, perm)


def setup_github_logging(args):
    gh_handler = None
    if args.comment_pr:
        github = GitHub("w3c", "web-platform-tests", args.gh_token)
        try:
            pr_number = int(args.comment_pr)
        except ValueError:
            pass
        else:
            gh_handler = GitHubCommentHandler(github, pr_number)
            gh_handler.setLevel(logging.INFO)
            logger.debug("Setting up GitHub logging")
            logger.addHandler(gh_handler)
    else:
        logger.warning("No PR number found; not posting to GitHub")
    return gh_handler


class pwd(object):
    def __init__(self, dir):
        self.dir = dir
        self.old_dir = None

    def __enter__(self):
        self.old_dir = os.path.abspath(os.curdir)
        os.chdir(self.dir)

    def __exit__(self, *args, **kwargs):
        os.chdir(self.old_dir)
        self.old_dir = None


def fetch_wpt_master():
    git = get_git_cmd(os.path.join(os.path.abspath(os.curdir), "w3c", "web-platform-tests"))
    git("fetch", "https://github.com/w3c/web-platform-tests.git", "master:master")


def get_sha1():
    git = get_git_cmd(os.path.join(os.path.abspath(os.curdir), "w3c", "web-platform-tests"))
    return git("rev-parse", "HEAD").strip()

def build_manifest():
    with pwd(os.path.join(os.path.abspath(os.curdir), "w3c", "web-platform-tests")):
        # TODO: Call the manifest code directly
        call("python", "manifest")


def install_wptrunner():
    call("git", "clone", "--depth=1", "https://github.com/w3c/wptrunner.git", "w3c/wptrunner")
    git = get_git_cmd(os.path.join(os.path.abspath(os.curdir), "w3c", "wptrunner"))
    git("submodule", "update", "--init", "--recursive")
    call("pip", "install", os.path.join("w3c", "wptrunner"))


def get_files_changed():
    root = os.path.abspath(os.curdir)
    git = get_git_cmd("%s/w3c/web-platform-tests" % root)
    branch_point = git("merge-base", "HEAD", "master").strip()
    logger.debug("Branch point from master: %s" % branch_point)
    logger.debug(git("log", "--oneline", "%s.." % branch_point))
    files = git("diff", "--name-only", "-z", "%s.." % branch_point)
    if not files:
        return []
    assert files[-1] == "\0"
    return ["%s/w3c/web-platform-tests/%s" % (root, item)
            for item in files[:-1].split("\0")]


def wptrunner_args(root, files_changed, iterations, browser):
    parser = wptcommandline.create_parser([browser.product])
    args = vars(parser.parse_args([]))
    wpt_root = os.path.join(root, "w3c", "web-platform-tests")
    args.update(browser.wptrunner_args(root))
    args.update({
        "tests_root": wpt_root,
        "metadata_root": wpt_root,
        "repeat": iterations,
        "config": "%s/w3c/wptrunner/wptrunner.default.ini" % root,
        "test_list": files_changed,
        "restart_on_unexpected": False,
        "pause_after_test": False
    })
    wptcommandline.check_args(args)
    return args


def setup_log_handler():
    global LogHandler

    class LogHandler(reader.LogHandler):
        def __init__(self):
            self.results = defaultdict(lambda: defaultdict(lambda: defaultdict(int)))

        def test_status(self, data):
            self.results[data["test"]][data.get("subtest")][data["status"]] += 1

        def test_end(self, data):
            self.results[data["test"]][None][data["status"]] += 1


def is_inconsistent(results_dict, iterations):
    return len(results_dict) > 1 or sum(results_dict.values()) != iterations


def err_string(results_dict, iterations):
    rv = []
    total_results = sum(results_dict.values())
    for key, value in sorted(results_dict.items()):
        rv.append("%s%s" %
                  (key, ": %s/%s" % (value, iterations) if value != iterations else ""))
    rv = ", ".join(rv)
    if total_results < iterations:
        rv.append("MISSING: %s/%s" % (iterations - total_results, iterations))
    if len(results_dict) > 1 or total_results != iterations:
        rv = "**%s**" % rv
    return rv


def process_results(log, iterations):
    inconsistent = []
    handler = LogHandler()
    reader.handle_log(reader.read(log), handler)
    results = handler.results
    for test, test_results in results.iteritems():
        for subtest, result in test_results.iteritems():
            if is_inconsistent(result, iterations):
                inconsistent.append((test, subtest, result))
    return results, inconsistent


def table(headings, data, log):
    cols = range(len(headings))
    assert all(len(item) == len(cols) for item in data)
    max_widths = reduce(lambda prev, cur: [(len(cur[i]) + 2)
                                           if (len(cur[i]) + 2) > prev[i]
                                           else prev[i]
                                           for i in cols],
                        data,
                        [len(item) + 2 for item in headings])
    log("|%s|" % "|".join(item.center(max_widths[i]) for i, item in enumerate(headings)))
    log("|%s|" % "|".join("-" * max_widths[i] for i in cols))
    for row in data:
        log("|%s|" % "|".join(" %s" % row[i].ljust(max_widths[i] - 1) for i in cols))
    log("")


def write_inconsistent(inconsistent, iterations):
    logger.error("## Unstable results ##\n")
    strings = [(test, subtest if subtest else "", err_string(results, iterations))
                for test, subtest, results in inconsistent]
    table(["Test", "Subtest", "Results"], strings, logger.error)


def write_results(results, iterations, comment_pr):
    logger.info("## All results ##\n")
    for test, test_results in results.iteritems():
        baseurl = "http://w3c-test.org/submissions"
        if "https" in os.path.splitext(test)[0].split(".")[1:]:
            baseurl = "https://w3c-test.org/submissions"
        pr_number = None
        if comment_pr:
            try:
                pr_number = int(comment_pr)
            except ValueError:
                pass
        if pr_number:
            logger.info("### [%s](%s/%s%s) ###" % (test, baseurl, pr_number, test))
        else:
            logger.info("### %s ###" % test)
        parent = test_results.pop(None)
        strings = [("", err_string(parent, iterations))]
        strings.extend(((subtest if subtest else "", err_string(results, iterations))
                        for subtest, results in test_results.iteritems()))
        table(["Subtest", "Results"], strings, logger.info)


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root",
                        action="store",
                        default=os.path.join(os.path.expanduser("~"), "build"),
                        help="Root path")
    parser.add_argument("--iterations",
                        action="store",
                        default=10,
                        type=int,
                        help="Number of times to run tests")
    parser.add_argument("--gh-token",
                        action="store",
                        default=os.environ.get("GH_TOKEN"),
                        help="OAuth token to use for accessing GitHub api")
    parser.add_argument("--comment-pr",
                        action="store",
                        default=os.environ.get("TRAVIS_PULL_REQUEST"),
                        help="PR to comment on with stability results")
    parser.add_argument("browser",
                        action="store",
                        help="Browser to run against")
    return parser


def main():
    retcode = 0
    parser = get_parser()
    args = parser.parse_args()

    if not os.path.exists(args.root):
        logger.critical("Root directory %s does not exist" % args.root)
        return 1

    os.chdir(args.root)

    if args.gh_token:
        gh_handler = setup_github_logging(args)
    else:
        logger.warning("Can't log to GitHub")
        gh_handler = None

    print >> sys.stderr, "travis_fold:start:browser_setup"
    logger.info("# %s #" % args.browser.title())

    browser_cls = {"firefox": Firefox,
                   "chrome": Chrome}.get(args.browser)
    if browser_cls is None:
        logger.critical("Unrecognised browser %s" % args.browser)
        return 1

    fetch_wpt_master()

    head_sha1 = get_sha1()
    logger.info("Testing revision %s" % head_sha1)

    # For now just pass the whole list of changed files to wptrunner and
    # assume that it will run everything that's actually a test
    files_changed = get_files_changed()

    if not files_changed:
        logger.info("No files changed")
        return 0

    build_manifest()
    install_wptrunner()
    do_delayed_imports()

    logger.debug("Files changed:\n%s" % "".join(" * %s\n" % item for item in files_changed))

    browser = browser_cls(args.gh_token)

    browser.install()
    browser.install_webdriver()

    kwargs = wptrunner_args(args.root,
                            files_changed,
                            args.iterations,
                            browser)

    print >> sys.stderr, "travis_fold:end:browser_setup"
    print >> sys.stderr, "travis_fold:start:running_tests"
    logger.info("Starting %i test iterations" % args.iterations)
    with open("raw.log", "wb") as log:
        wptrunner.setup_logging(kwargs,
                                {"raw": log})
        # Setup logging for wptrunner that keeps process output and
        # warning+ level logs only
        wptrunner.logger.add_handler(
            LogActionFilter(
                LogLevelFilter(
                    StreamHandler(
                        sys.stdout,
                        TbplFormatter()
                    ),
                    "WARNING"),
                ["log", "process_output"]))

        wptrunner.run_tests(**kwargs)

    with open("raw.log", "rb") as log:
        results, inconsistent = process_results(log, args.iterations)

    print >> sys.stderr, "travis_fold:end:running_tests"

    if results:
        if inconsistent:
            write_inconsistent(inconsistent, args.iterations)
            retcode = 2
        else:
            logger.info("All results were stable\n")
        print >> sys.stderr, "travis_fold:start:full_results"
        write_results(results, args.iterations, args.comment_pr)
        print >> sys.stderr, "travis_fold:end:full_results"
    else:
        logger.info("No tests run.")

    try:
        if gh_handler:
            gh_handler.send()
    except Exception:
        logger.error(traceback.format_exc())
    return retcode


if __name__ == "__main__":
    try:
        retcode = main()
    except:
        raise
    else:
        sys.exit(retcode)
