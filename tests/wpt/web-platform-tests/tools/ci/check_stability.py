from __future__ import print_function

import argparse
import logging
import os
import subprocess
import sys
from ConfigParser import SafeConfigParser

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))
sys.path.insert(0, wpt_root)

from tools.wpt import testfiles
from tools.wpt.testfiles import get_git_cmd
from tools.wpt.virtualenv import Virtualenv
from tools.wpt.utils import Kwargs
from tools.wpt.run import create_parser, setup_wptrunner
from tools.wpt import markdown
from tools import localpaths

logger = None
stability_run, write_inconsistent, write_results = None, None, None
wptrunner = None

def setup_logging():
    """Set up basic debug logger."""
    global logger
    logger = logging.getLogger(here)
    handler = logging.StreamHandler(sys.stdout)
    formatter = logging.Formatter(logging.BASIC_FORMAT, None)
    handler.setFormatter(formatter)
    logger.addHandler(handler)
    logger.setLevel(logging.DEBUG)


def do_delayed_imports():
    global stability_run, write_inconsistent, write_results, wptrunner
    from tools.wpt.stability import run as stability_run
    from tools.wpt.stability import write_inconsistent, write_results
    from wptrunner import wptrunner


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

def fetch_wpt(user, *args):
    git = get_git_cmd(wpt_root)
    git("fetch", "https://github.com/%s/web-platform-tests.git" % user, *args)


def get_sha1():
    """ Get and return sha1 of current git branch HEAD commit."""
    git = get_git_cmd(wpt_root)
    return git("rev-parse", "HEAD").strip()


def deepen_checkout(user):
    """Convert from a shallow checkout to a full one"""
    fetch_args = [user, "+refs/heads/*:refs/remotes/origin/*"]
    if os.path.exists(os.path.join(wpt_root, ".git", "shallow")):
        fetch_args.insert(1, "--unshallow")
    fetch_wpt(*fetch_args)


def get_parser():
    """Create and return script-specific argument parser."""
    description = """Detect instabilities in new tests by executing tests
    repeatedly and comparing results between executions."""
    parser = argparse.ArgumentParser(description=description)
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
    parser.add_argument("--metadata",
                        dest="metadata_root",
                        action="store",
                        default=wpt_root,
                        help="Directory that will contain MANIFEST.json")
    parser.add_argument("--config-file",
                        action="store",
                        type=str,
                        help="Location of ini-formatted configuration file",
                        default="check_stability.ini")
    parser.add_argument("--rev",
                        action="store",
                        default=None,
                        help="Commit range to use")
    return parser


def pr():
    pr = os.environ.get("TRAVIS_PULL_REQUEST", "false")
    return pr if pr != "false" else None


def get_changed_files(manifest_path, rev, ignore_changes, skip_tests):
    if not rev:
        branch_point = testfiles.branch_point()
        revish = "%s..HEAD" % branch_point
    else:
        revish = rev

    files_changed, files_ignored = testfiles.files_changed(revish, ignore_changes)

    if files_ignored:
        logger.info("Ignoring %s changed files:\n%s" %
                    (len(files_ignored), "".join(" * %s\n" % item for item in files_ignored)))

    tests_changed, files_affected = testfiles.affected_testfiles(files_changed, skip_tests,
                                                                 manifest_path=manifest_path)

    return tests_changed, files_affected


def main():
    """Perform check_stability functionality and return exit code."""

    venv = Virtualenv(os.environ.get("VIRTUAL_ENV", os.path.join(wpt_root, "_venv")))
    venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", "requirements.txt"))

    args, wpt_args = get_parser().parse_known_args()
    return run(venv, wpt_args, **vars(args))


def run(venv, wpt_args, **kwargs):
    do_delayed_imports()

    retcode = 0

    wpt_args = create_parser().parse_args(wpt_args)

    with open(kwargs["config_file"], 'r') as config_fp:
        config = SafeConfigParser()
        config.readfp(config_fp)
        skip_tests = config.get("file detection", "skip_tests").split()
        ignore_changes = set(config.get("file detection", "ignore_changes").split())

    if kwargs["output_bytes"] is not None:
        replace_streams(kwargs["output_bytes"],
                        "Log reached capacity (%s bytes); output disabled." % kwargs["output_bytes"])


    wpt_args.metadata_root = kwargs["metadata_root"]
    try:
        os.makedirs(wpt_args.metadata_root)
    except OSError:
        pass

    setup_logging()

    pr_number = pr()

    with TravisFold("browser_setup"):
        logger.info(markdown.format_comment_title(wpt_args.product))

        if pr is not None:
            deepen_checkout(kwargs["user"])

        # Ensure we have a branch called "master"
        fetch_wpt(kwargs["user"], "master:master")

        head_sha1 = get_sha1()
        logger.info("Testing web-platform-tests at revision %s" % head_sha1)

        wpt_kwargs = Kwargs(vars(wpt_args))

        if not wpt_kwargs["test_list"]:
            manifest_path = os.path.join(wpt_kwargs["metadata_root"], "MANIFEST.json")
            tests_changed, files_affected = get_changed_files(manifest_path, kwargs["rev"],
                                                              ignore_changes, skip_tests)

            if not (tests_changed or files_affected):
                logger.info("No tests changed")
                return 0

            if tests_changed:
                logger.debug("Tests changed:\n%s" % "".join(" * %s\n" % item for item in tests_changed))

            if files_affected:
                logger.debug("Affected tests:\n%s" % "".join(" * %s\n" % item for item in files_affected))

            wpt_kwargs["test_list"] = list(tests_changed | files_affected)

        do_delayed_imports()

        wpt_kwargs["stability"] = True
        wpt_kwargs["prompt"] = False
        wpt_kwargs["install_browser"] = True
        wpt_kwargs["install"] = wpt_kwargs["product"].split(":")[0] == "firefox"

        wpt_kwargs = setup_wptrunner(venv, **wpt_kwargs)

        logger.info("Using binary %s" % wpt_kwargs["binary"])


    with TravisFold("running_tests"):
        logger.info("Starting tests")


        wpt_logger = wptrunner.logger
        iterations, results, inconsistent = stability_run(venv, wpt_logger, **wpt_kwargs)

    if results:
        if inconsistent:
            write_inconsistent(logger.error, inconsistent, iterations)
            retcode = 2
        else:
            logger.info("All results were stable\n")
        with TravisFold("full_results"):
            write_results(logger.info, results, iterations,
                          pr_number=pr_number,
                          use_details=True)
    else:
        logger.info("No tests run.")
        # Be conservative and only return errors when we know for sure tests are changed.
        if tests_changed:
            retcode = 3

    return retcode


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        import traceback
        traceback.print_exc()
        sys.exit(1)
