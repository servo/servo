from __future__ import print_function

import argparse
import logging
import os
import re
import stat
import subprocess
import sys
import tarfile
import zipfile
from ConfigParser import RawConfigParser, SafeConfigParser
from abc import ABCMeta, abstractmethod
from cStringIO import StringIO as CStringIO
from collections import defaultdict, OrderedDict
from io import BytesIO, StringIO

import requests

BaseHandler = None
LogActionFilter = None
LogHandler = None
LogLevelFilter = None
StreamHandler = None
TbplFormatter = None
manifest = None
reader = None
wptcommandline = None
wptrunner = None
wpt_root = None
wptrunner_root = None

logger = None


def do_delayed_imports():
    """Import and set up modules only needed if execution gets to this point."""
    global BaseHandler
    global LogLevelFilter
    global StreamHandler
    global TbplFormatter
    global manifest
    global reader
    global wptcommandline
    global wptrunner
    from mozlog import reader
    from mozlog.formatters import TbplFormatter
    from mozlog.handlers import BaseHandler, LogLevelFilter, StreamHandler
    from tools.manifest import manifest
    from wptrunner import wptcommandline, wptrunner
    setup_log_handler()
    setup_action_filter()


def setup_logging():
    """Set up basic debug logger."""
    handler = logging.StreamHandler(sys.stdout)
    formatter = logging.Formatter(logging.BASIC_FORMAT, None)
    handler.setFormatter(formatter)
    logger.addHandler(handler)
    logger.setLevel(logging.DEBUG)


def setup_action_filter():
    """Create global LogActionFilter class as part of deferred module load."""
    global LogActionFilter

    class LogActionFilter(BaseHandler):

        """Handler that filters out messages not of a given set of actions.

        Subclasses BaseHandler.

        :param inner: Handler to use for messages that pass this filter
        :param actions: List of actions for which to fire the handler
        """

        def __init__(self, inner, actions):
            """Extend BaseHandler and set inner and actions props on self."""
            BaseHandler.__init__(self, inner)
            self.inner = inner
            self.actions = actions

        def __call__(self, item):
            """Invoke handler if action is in list passed as constructor param."""
            if item["action"] in self.actions:
                return self.inner(item)


class TravisFold(object):

    """Context for TravisCI folding mechanism. Subclasses object.

    See: https://blog.travis-ci.com/2013-05-22-improving-build-visibility-log-folds/
    """

    def __init__(self, name):
        """Register TravisCI folding section name."""
        self.name = name

    def __enter__(self):
        """Emit fold start syntax."""
        print("travis_fold:start:%s" % self.name, file=sys.stderr)

    def __exit__(self, type, value, traceback):
        """Emit fold end syntax."""
        print("travis_fold:end:%s" % self.name, file=sys.stderr)


class FilteredIO(object):
    """Wrap a file object, invoking the provided callback for every call to
    `write` and only proceeding with the operation when that callback returns
    True."""
    def __init__(self, original, on_write):
        self.original = original
        self.on_write = on_write

    def __getattr__(self, name):
        return getattr(self.original, name)

    def disable(self):
        self.write = lambda msg: None

    def write(self, msg):
        encoded = msg.encode("utf8", "backslashreplace").decode("utf8")
        if self.on_write(self.original, encoded) is True:
            self.original.write(encoded)


def replace_streams(capacity, warning_msg):
    # Value must be boxed to support modification from inner function scope
    count = [0]
    capacity -= 2 + len(warning_msg)
    stderr = sys.stderr

    def on_write(handle, msg):
        length = len(msg)
        count[0] += length

        if count[0] > capacity:
            wrapped_stdout.disable()
            wrapped_stderr.disable()
            handle.write(msg[0:capacity - count[0]])
            handle.flush()
            stderr.write("\n%s\n" % warning_msg)
            return False

        return True

    # Store local references to the replaced streams to guard against the case
    # where other code replace the global references.
    sys.stdout = wrapped_stdout = FilteredIO(sys.stdout, on_write)
    sys.stderr = wrapped_stderr = FilteredIO(sys.stderr, on_write)


class Browser(object):
    __metaclass__ = ABCMeta

    @abstractmethod
    def install(self):
        return NotImplemented

    @abstractmethod
    def install_webdriver(self):
        return NotImplemented

    @abstractmethod
    def version(self):
        return NotImplemented

    @abstractmethod
    def wptrunner_args(self):
        return NotImplemented


class Firefox(Browser):
    """Firefox-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "firefox"
    binary = "%s/firefox/firefox"
    platform_ini = "%s/firefox/platform.ini"

    def __init__(self, **kwargs):
        pass

    def install(self):
        """Install Firefox."""
        call("pip", "install", "-r", os.path.join(wptrunner_root, "requirements_firefox.txt"))
        index = get("https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/")
        latest = re.compile("<a[^>]*>(firefox-\d+\.\d(?:\w\d)?.en-US.linux-x86_64\.tar\.bz2)</a>")
        filename = latest.search(index.text).group(1)
        resp = get("https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/%s" %
                   filename)
        untar(resp.raw)

        if not os.path.exists("profiles"):
            os.mkdir("profiles")
        with open(os.path.join("profiles", "prefs_general.js"), "wb") as f:
            resp = get("https://hg.mozilla.org/mozilla-central/raw-file/tip/testing/profiles/prefs_general.js")
            f.write(resp.content)
        call("pip", "install", "-r", os.path.join(wptrunner_root, "requirements_firefox.txt"))

    def _latest_geckodriver_version(self):
        """Get and return latest version number for geckodriver."""
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
        """Install latest Geckodriver."""
        version = self._latest_geckodriver_version()
        logger.debug("Latest geckodriver release %s" % version)
        url = "https://github.com/mozilla/geckodriver/releases/download/%s/geckodriver-%s-linux64.tar.gz" % (version, version)
        untar(get(url).raw)

    def version(self, root):
        """Retrieve the release version of the installed browser."""
        platform_info = RawConfigParser()

        with open(self.platform_ini % root, "r") as fp:
            platform_info.readfp(BytesIO(fp.read()))
            return "BuildID %s; SourceStamp %s" % (
                platform_info.get("Build", "BuildID"),
                platform_info.get("Build", "SourceStamp"))

    def wptrunner_args(self, root):
        """Return Firefox-specific wpt-runner arguments."""
        return {
            "product": "firefox",
            "binary": self.binary % root,
            "certutil_binary": "certutil",
            "webdriver_binary": "%s/geckodriver" % root,
            "prefs_root": "%s/profiles" % root,
        }


class Chrome(Browser):
    """Chrome-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "chrome"
    binary = "/usr/bin/google-chrome"

    def __init__(self, **kwargs):
        pass

    def install(self):
        """Install Chrome."""

        # Installing the Google Chrome browser requires administrative
        # privileges, so that installation is handled by the invoking script.

        call("pip", "install", "-r", os.path.join(wptrunner_root, "requirements_chrome.txt"))

    def install_webdriver(self):
        """Install latest Webdriver."""
        latest = get("http://chromedriver.storage.googleapis.com/LATEST_RELEASE").text.strip()
        url = "http://chromedriver.storage.googleapis.com/%s/chromedriver_linux64.zip" % latest
        unzip(get(url).raw)
        st = os.stat('chromedriver')
        os.chmod('chromedriver', st.st_mode | stat.S_IEXEC)

    def version(self, root):
        """Retrieve the release version of the installed browser."""
        output = call(self.binary, "--version")
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)

    def wptrunner_args(self, root):
        """Return Chrome-specific wpt-runner arguments."""
        return {
            "product": "chrome",
            "binary": self.binary,
            "webdriver_binary": "%s/chromedriver" % root,
            "test_types": ["testharness", "reftest"]
        }


class Sauce(Browser):
    """Sauce-specific interface.

    Includes installation and wptrunner setup methods.
    """

    product = "sauce"

    def __init__(self, **kwargs):
        browser = kwargs["product"].split(":")
        self.browser_name = browser[1]
        self.browser_version = browser[2]
        self.sauce_platform = kwargs["sauce_platform"]
        self.sauce_build = kwargs["sauce_build_number"]
        self.sauce_key = kwargs["sauce_key"]
        self.sauce_user = kwargs["sauce_user"]
        self.sauce_build_tags = kwargs["sauce_build_tags"]
        self.sauce_tunnel_id = kwargs["sauce_tunnel_identifier"]

    def install(self):
        """Install sauce selenium python deps."""
        call("pip", "install", "-r", os.path.join(wptrunner_root, "requirements_sauce.txt"))

    def install_webdriver(self):
        """No need to install webdriver locally."""
        pass

    def version(self, root):
        """Retrieve the release version of the browser under test."""
        return self.browser_version

    def wptrunner_args(self, root):
        """Return Sauce-specific wptrunner arguments."""
        return {
            "product": "sauce",
            "sauce_browser": self.browser_name,
            "sauce_build": self.sauce_build,
            "sauce_key": self.sauce_key,
            "sauce_platform": self.sauce_platform,
            "sauce_tags": self.sauce_build_tags,
            "sauce_tunnel_id": self.sauce_tunnel_id,
            "sauce_user": self.sauce_user,
            "sauce_version": self.browser_version,
            "test_types": ["testharness", "reftest"]
        }


def get(url):
    """Issue GET request to a given URL and return the response."""
    logger.debug("GET %s" % url)
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    return resp


def call(*args):
    """Log terminal command, invoke it as a subprocess.

    Returns a bytestring of the subprocess output if no error.
    """
    logger.debug("%s" % " ".join(args))
    try:
        return subprocess.check_output(args)
    except subprocess.CalledProcessError as e:
        logger.critical("%s exited with return code %i" %
                        (e.cmd, e.returncode))
        logger.critical(e.output)
        raise


def get_git_cmd(repo_path):
    """Create a function for invoking git commands as a subprocess."""
    def git(cmd, *args):
        full_cmd = ["git", cmd] + list(args)
        try:
            logger.debug(" ".join(full_cmd))
            return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT).strip()
        except subprocess.CalledProcessError as e:
            logger.error("Git command exited with status %i" % e.returncode)
            logger.error(e.output)
            sys.exit(1)
    return git


def seekable(fileobj):
    """Attempt to use file.seek on given file, with fallbacks."""
    try:
        fileobj.seek(fileobj.tell())
    except Exception:
        return CStringIO(fileobj.read())
    else:
        return fileobj


def untar(fileobj):
    """Extract tar archive."""
    logger.debug("untar")
    fileobj = seekable(fileobj)
    with tarfile.open(fileobj=fileobj) as tar_data:
        tar_data.extractall()


def unzip(fileobj):
    """Extract zip archive."""
    logger.debug("unzip")
    fileobj = seekable(fileobj)
    with zipfile.ZipFile(fileobj) as zip_data:
        for info in zip_data.infolist():
            zip_data.extract(info)
            perm = info.external_attr >> 16 & 0x1FF
            os.chmod(info.filename, perm)


class pwd(object):
    """Create context for temporarily changing present working directory."""
    def __init__(self, dir):
        self.dir = dir
        self.old_dir = None

    def __enter__(self):
        self.old_dir = os.path.abspath(os.curdir)
        os.chdir(self.dir)

    def __exit__(self, *args, **kwargs):
        os.chdir(self.old_dir)
        self.old_dir = None


def fetch_wpt(user, *args):
    git = get_git_cmd(wpt_root)
    git("fetch", "https://github.com/%s/web-platform-tests.git" % user, *args)


def get_sha1():
    """ Get and return sha1 of current git branch HEAD commit."""
    git = get_git_cmd(wpt_root)
    return git("rev-parse", "HEAD").strip()


def build_manifest():
    """Build manifest of all files in web-platform-tests"""
    with pwd(wpt_root):
        # TODO: Call the manifest code directly
        call("python", "manifest")


def install_wptrunner():
    """Install wptrunner."""
    call("pip", "install", wptrunner_root)


def get_branch_point(user):
    git = get_git_cmd(wpt_root)
    if os.environ.get("TRAVIS_PULL_REQUEST", "false") != "false":
        # This is a PR, so the base branch is in TRAVIS_BRANCH
        travis_branch = os.environ.get("TRAVIS_BRANCH")
        assert travis_branch, "TRAVIS_BRANCH environment variable is defined"
        branch_point = git("rev-parse", travis_branch)
    else:
        # Otherwise we aren't on a PR, so we try to find commits that are only in the
        # current branch c.f.
        # http://stackoverflow.com/questions/13460152/find-first-ancestor-commit-in-another-branch
        head = git("rev-parse", "HEAD")
        # To do this we need all the commits in the local copy
        fetch_args = [user, "+refs/heads/*:refs/remotes/origin/*"]
        if os.path.exists(os.path.join(wpt_root, ".git", "shallow")):
            fetch_args.insert(1, "--unshallow")
        fetch_wpt(*fetch_args)
        not_heads = [item for item in git("rev-parse", "--not", "--all").split("\n")
                     if item.strip() and not head in item]
        commits = git("rev-list", "HEAD", *not_heads).split("\n")
        branch_point = None
        if len(commits):
            first_commit = commits[-1]
            if first_commit:
                branch_point = git("rev-parse", first_commit + "^")

        # The above heuristic will fail in the following cases:
        #
        # - The current branch has fallen behind the version retrieved via the above
        #   `fetch` invocation
        # - Changes on the current branch were rebased and therefore do not exist on any
        #   other branch. This will result in the selection of a commit that is earlier
        #   in the history than desired (as determined by calculating the later of the
        #   branch point and the merge base)
        #
        # In either case, fall back to using the merge base as the branch point.
        merge_base = git("merge-base", "HEAD", "origin/master")
        if (branch_point is None or
            (branch_point != merge_base and
             not git("log", "--oneline", "%s..%s" % (merge_base, branch_point)).strip())):
            logger.debug("Using merge-base as the branch point")
            branch_point = merge_base
        else:
            logger.debug("Using first commit on another branch as the branch point")

    logger.debug("Branch point from master: %s" % branch_point)
    return branch_point


def get_files_changed(branch_point, ignore_changes):
    """Get and return files changed since current branch diverged from master,
    excluding those that are located within any directory specifed by
    `ignore_changes`."""
    root = os.path.abspath(os.curdir)
    git = get_git_cmd(wpt_root)
    files = git("diff", "--name-only", "-z", "%s..." % branch_point)
    if not files:
        return [], []
    assert files[-1] == "\0"

    changed = []
    ignored = []
    for item in files[:-1].split("\0"):
        fullpath = os.path.join(wpt_root, item)
        topmost_dir = item.split(os.sep, 1)[0]
        if topmost_dir in ignore_changes:
            ignored.append(fullpath)
        else:
            changed.append(fullpath)

    return changed, ignored


def _in_repo_root(full_path):
    rel_path = os.path.relpath(full_path, wpt_root)
    path_components = rel_path.split(os.sep)
    return len(path_components) < 2


def get_affected_testfiles(files_changed, skip_tests):
    """Determine and return list of test files that reference changed files."""
    affected_testfiles = set()
    # Exclude files that are in the repo root, because
    # they are not part of any test.
    files_changed = [f for f in files_changed if not _in_repo_root(f)]
    nontests_changed = set(files_changed)
    manifest_file = os.path.join(wpt_root, "MANIFEST.json")
    test_types = ["testharness", "reftest", "wdspec"]

    wpt_manifest = manifest.load(wpt_root, manifest_file)

    support_files = {os.path.join(wpt_root, path)
                     for _, path, _ in wpt_manifest.itertypes("support")}
    wdspec_test_files = {os.path.join(wpt_root, path)
                         for _, path, _ in wpt_manifest.itertypes("wdspec")}
    test_files = {os.path.join(wpt_root, path)
                  for _, path, _ in wpt_manifest.itertypes(*test_types)}

    nontests_changed = nontests_changed.intersection(support_files)

    nontest_changed_paths = set()
    for full_path in nontests_changed:
        rel_path = os.path.relpath(full_path, wpt_root)
        path_components = rel_path.split(os.sep)
        top_level_subdir = path_components[0]
        if top_level_subdir in skip_tests:
            continue
        repo_path = "/" + os.path.relpath(full_path, wpt_root).replace(os.path.sep, "/")
        nontest_changed_paths.add((full_path, repo_path))

    def affected_by_wdspec(test):
        affected = False
        if test in wdspec_test_files:
            for support_full_path, _ in nontest_changed_paths:
                # parent of support file or of "support" directory
                parent = os.path.dirname(support_full_path)
                if os.path.basename(parent) == "support":
                    parent = os.path.dirname(parent)
                relpath = os.path.relpath(test, parent)
                if not relpath.startswith(os.pardir):
                    # testfile is in subtree of support file
                    affected = True
                    break
        return affected

    for root, dirs, fnames in os.walk(wpt_root):
        # Walk top_level_subdir looking for test files containing either the
        # relative filepath or absolute filepath to the changed files.
        if root == wpt_root:
            for dir_name in skip_tests:
                dirs.remove(dir_name)
        for fname in fnames:
            test_full_path = os.path.join(root, fname)
            # Skip any file that's not a test file.
            if test_full_path not in test_files:
                continue
            if affected_by_wdspec(test_full_path):
                affected_testfiles.add(test_full_path)
                continue

            with open(test_full_path, "rb") as fh:
                file_contents = fh.read()
                if file_contents.startswith("\xfe\xff"):
                    file_contents = file_contents.decode("utf-16be")
                elif file_contents.startswith("\xff\xfe"):
                    file_contents = file_contents.decode("utf-16le")
                for full_path, repo_path in nontest_changed_paths:
                    rel_path = os.path.relpath(full_path, root).replace(os.path.sep, "/")
                    if rel_path in file_contents or repo_path in file_contents:
                        affected_testfiles.add(test_full_path)
                        continue
    return affected_testfiles


def wptrunner_args(root, files_changed, iterations, browser):
    """Derive and return arguments for wpt-runner."""
    parser = wptcommandline.create_parser([browser.product])
    args = vars(parser.parse_args([]))
    args.update(browser.wptrunner_args(root))
    args.update({
        "tests_root": wpt_root,
        "metadata_root": wpt_root,
        "repeat": iterations,
        "config": "%s//wptrunner.default.ini" % (wptrunner_root),
        "test_list": files_changed,
        "restart_on_unexpected": False,
        "pause_after_test": False
    })
    wptcommandline.check_args(args)
    return args


def setup_log_handler():
    """Set up LogHandler class as part of deferred module load."""
    global LogHandler

    class LogHandler(reader.LogHandler):

        """Handle updating test and subtest status in log.

        Subclasses reader.LogHandler.
        """
        def __init__(self):
            self.results = OrderedDict()

        def find_or_create_test(self, data):
            test_name = data["test"]
            if self.results.get(test_name):
                return self.results[test_name]

            test = {
                "subtests": OrderedDict(),
                "status": defaultdict(int)
            }
            self.results[test_name] = test
            return test

        def find_or_create_subtest(self, data):
            test = self.find_or_create_test(data)
            subtest_name = data["subtest"]

            if test["subtests"].get(subtest_name):
                return test["subtests"][subtest_name]

            subtest = {
                "status": defaultdict(int),
                "messages": set()
            }
            test["subtests"][subtest_name] = subtest

            return subtest

        def test_status(self, data):
            subtest = self.find_or_create_subtest(data)
            subtest["status"][data["status"]] += 1
            if data.get("message"):
                subtest["messages"].add(data["message"])

        def test_end(self, data):
            test = self.find_or_create_test(data)
            test["status"][data["status"]] += 1


def is_inconsistent(results_dict, iterations):
    """Return whether or not a single test is inconsistent."""
    return len(results_dict) > 1 or sum(results_dict.values()) != iterations


def err_string(results_dict, iterations):
    """Create and return string with errors from test run."""
    rv = []
    total_results = sum(results_dict.values())
    for key, value in sorted(results_dict.items()):
        rv.append("%s%s" %
                  (key, ": %s/%s" % (value, iterations) if value != iterations else ""))
    if total_results < iterations:
        rv.append("MISSING: %s/%s" % (iterations - total_results, iterations))
    rv = ", ".join(rv)
    if is_inconsistent(results_dict, iterations):
        rv = "**%s**" % rv
    return rv


def process_results(log, iterations):
    """Process test log and return overall results and list of inconsistent tests."""
    inconsistent = []
    handler = LogHandler()
    reader.handle_log(reader.read(log), handler)
    results = handler.results
    for test_name, test in results.iteritems():
        if is_inconsistent(test["status"], iterations):
            inconsistent.append((test_name, None, test["status"], []))
        for subtest_name, subtest in test["subtests"].iteritems():
            if is_inconsistent(subtest["status"], iterations):
                inconsistent.append((test_name, subtest_name, subtest["status"], subtest["messages"]))
    return results, inconsistent


def format_comment_title(product):
    """Produce a Markdown-formatted string based on a given "product"--a string
    containing a browser identifier optionally followed by a colon and a
    release channel. (For example: "firefox" or "chrome:dev".) The generated
    title string is used both to create new comments and to locate (and
    subsequently update) previously-submitted comments."""
    parts = product.split(":")
    title = parts[0].title()

    if len(parts) > 1:
       title += " (%s)" % parts[1]

    return "# %s #" % title


def markdown_adjust(s):
    """Escape problematic markdown sequences."""
    s = s.replace('\t', u'\\t')
    s = s.replace('\n', u'\\n')
    s = s.replace('\r', u'\\r')
    s = s.replace('`',  u'')
    s = s.replace('|', u'\\|')
    return s


def table(headings, data, log):
    """Create and log data to specified logger in tabular format."""
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
    """Output inconsistent tests to logger.error."""
    logger.error("## Unstable results ##\n")
    strings = [(
        "`%s`" % markdown_adjust(test),
        ("`%s`" % markdown_adjust(subtest)) if subtest else "",
        err_string(results, iterations),
        ("`%s`" % markdown_adjust(";".join(messages))) if len(messages) else ""
    )
               for test, subtest, results, messages in inconsistent]
    table(["Test", "Subtest", "Results", "Messages"], strings, logger.error)


def write_results(results, iterations, comment_pr):
    """Output all test results to logger.info."""
    pr_number = None
    if comment_pr:
        try:
            pr_number = int(comment_pr)
        except ValueError:
            pass
    logger.info("## All results ##\n")
    if pr_number:
        logger.info("<details>\n")
        logger.info("<summary>%i %s ran</summary>\n\n" % (len(results),
                                                          "tests" if len(results) > 1
                                                          else "test"))

    for test_name, test in results.iteritems():
        baseurl = "http://w3c-test.org/submissions"
        if "https" in os.path.splitext(test_name)[0].split(".")[1:]:
            baseurl = "https://w3c-test.org/submissions"
        if pr_number:
            logger.info("<details>\n")
            logger.info('<summary><a href="%s/%s%s">%s</a></summary>\n\n' %
                        (baseurl, pr_number, test_name, test_name))
        else:
            logger.info("### %s ###" % test_name)
        strings = [("", err_string(test["status"], iterations), "")]

        strings.extend(((
            ("`%s`" % markdown_adjust(subtest_name)) if subtest else "",
            err_string(subtest["status"], iterations),
            ("`%s`" % markdown_adjust(';'.join(subtest["messages"]))) if len(subtest["messages"]) else ""
        ) for subtest_name, subtest in test["subtests"].items()))
        table(["Subtest", "Results", "Messages"], strings, logger.info)
        if pr_number:
            logger.info("</details>\n")

    if pr_number:
        logger.info("</details>\n")


def get_parser():
    """Create and return script-specific argument parser."""
    description = """Detect instabilities in new tests by executing tests
    repeatedly and comparing results between executions."""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--root",
                        action="store",
                        default=os.path.join(os.path.expanduser("~"), "build"),
                        help="Root path")
    parser.add_argument("--iterations",
                        action="store",
                        default=10,
                        type=int,
                        help="Number of times to run tests")
    parser.add_argument("--comment-pr",
                        action="store",
                        default=os.environ.get("TRAVIS_PULL_REQUEST"),
                        help="PR to comment on with stability results")
    parser.add_argument("--user",
                        action="store",
                        # Travis docs say do not depend on USER env variable.
                        # This is a workaround to get what should be the same value
                        default=os.environ.get("TRAVIS_REPO_SLUG", "w3c").split('/')[0],
                        help="Travis user name")
    parser.add_argument("--output-bytes",
                        action="store",
                        type=int,
                        help="Maximum number of bytes to write to standard output/error")
    parser.add_argument("--config-file",
                        action="store",
                        type=str,
                        help="Location of ini-formatted configuration file",
                        default="check_stability.ini")
    parser.add_argument("--sauce-platform",
                        action="store",
                        default=os.environ.get("PLATFORM"),
                        help="Sauce Labs OS")
    parser.add_argument("--sauce-build-number",
                        action="store",
                        default=os.environ.get("TRAVIS_BUILD_NUMBER"),
                        help="Sauce Labs build identifier")
    parser.add_argument("--sauce-build-tags",
                        action="store", nargs="*",
                        default=[os.environ.get("TRAVIS_PYTHON_VERSION")],
                        help="Sauce Labs build tag")
    parser.add_argument("--sauce-tunnel-identifier",
                        action="store",
                        default=os.environ.get("TRAVIS_JOB_NUMBER"),
                        help="Sauce Connect tunnel identifier")
    parser.add_argument("--sauce-user",
                        action="store",
                        default=os.environ.get("SAUCE_USERNAME"),
                        help="Sauce Labs user name")
    parser.add_argument("--sauce-key",
                        action="store",
                        default=os.environ.get("SAUCE_ACCESS_KEY"),
                        help="Sauce Labs access key")
    parser.add_argument("product",
                        action="store",
                        help="Product to run against (`browser-name` or 'browser-name:channel')")
    return parser


def main():
    """Perform check_stability functionality and return exit code."""
    global wpt_root
    global wptrunner_root
    global logger

    retcode = 0
    parser = get_parser()
    args = parser.parse_args()

    with open(args.config_file, 'r') as config_fp:
        config = SafeConfigParser()
        config.readfp(config_fp)
        skip_tests = config.get("file detection", "skip_tests").split()
        ignore_changes = set(config.get("file detection", "ignore_changes").split())

    if args.output_bytes is not None:
        replace_streams(args.output_bytes,
                        "Log reached capacity (%s bytes); output disabled." % args.output_bytes)

    logger = logging.getLogger(os.path.splitext(__file__)[0])
    setup_logging()

    wpt_root = os.path.abspath(os.curdir)
    wptrunner_root = os.path.normpath(os.path.join(wpt_root, "tools", "wptrunner"))

    if not os.path.exists(args.root):
        logger.critical("Root directory %s does not exist" % args.root)
        return 1

    os.chdir(args.root)
    browser_name = args.product.split(":")[0]

    if browser_name == "sauce" and not args.sauce_key:
        logger.warning("Cannot run tests on Sauce Labs. No access key.")
        return retcode

    with TravisFold("browser_setup"):
        logger.info(format_comment_title(args.product))

        browser_cls = {"firefox": Firefox,
                       "chrome": Chrome,
                       "sauce": Sauce}.get(browser_name)
        if browser_cls is None:
            logger.critical("Unrecognised browser %s" % browser_name)
            return 1

        fetch_wpt(args.user, "master:master")

        head_sha1 = get_sha1()
        logger.info("Testing web-platform-tests at revision %s" % head_sha1)

        branch_point = get_branch_point(args.user)

        # For now just pass the whole list of changed files to wptrunner and
        # assume that it will run everything that's actually a test
        files_changed, files_ignored = get_files_changed(branch_point, ignore_changes)

        if files_ignored:
            logger.info("Ignoring %s changed files:\n%s" % (len(files_ignored),
                                                            "".join(" * %s\n" % item for item in files_ignored)))

        if not files_changed:
            logger.info("No files changed")
            return 0

        build_manifest()
        install_wptrunner()
        do_delayed_imports()

        browser = browser_cls(**vars(args))
        browser.install()
        browser.install_webdriver()

        try:
            version = browser.version(args.root)
        except Exception, e:
            version = "unknown (error: %s)" % e
        logger.info("Using browser at version %s", version)

        logger.debug("Files changed:\n%s" % "".join(" * %s\n" % item for item in files_changed))

        affected_testfiles = get_affected_testfiles(files_changed, skip_tests)

        logger.debug("Affected tests:\n%s" % "".join(" * %s\n" % item for item in affected_testfiles))

        files_changed.extend(affected_testfiles)

        kwargs = wptrunner_args(args.root,
                                files_changed,
                                args.iterations,
                                browser)

    with TravisFold("running_tests"):
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

    if results:
        if inconsistent:
            write_inconsistent(inconsistent, args.iterations)
            retcode = 2
        else:
            logger.info("All results were stable\n")
        with TravisFold("full_results"):
            write_results(results, args.iterations, args.comment_pr)
    else:
        logger.info("No tests run.")

    return retcode


if __name__ == "__main__":
    try:
        retcode = main()
    except:
        import traceback
        traceback.print_exc()
        sys.exit(1)
    else:
        sys.exit(retcode)
