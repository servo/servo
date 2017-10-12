import argparse
import itertools
import logging
import os
import subprocess
import sys

from ..manifest import manifest, update

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

logger = logging.getLogger()


def get_git_cmd(repo_path):
    """Create a function for invoking git commands as a subprocess."""
    def git(cmd, *args):
        full_cmd = ["git", cmd] + list(item.decode("utf8") if isinstance(item, bytes) else item for item in args)
        try:
            logger.debug(" ".join(full_cmd))
            return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT).decode("utf8").strip()
        except subprocess.CalledProcessError as e:
            logger.error("Git command exited with status %i" % e.returncode)
            logger.error(e.output)
            sys.exit(1)
    return git


def branch_point():
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
        not_heads = [item for item in git("rev-parse", "--not", "--all").split("\n")
                     if item.strip() and head not in item]
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


def files_changed(revish, ignore_dirs=None, include_uncommitted=False, include_new=False):
    """Get and return files changed since current branch diverged from master,
    excluding those that are located within any directory specifed by
    `ignore_changes`."""
    if ignore_dirs is None:
        ignore_dirs = []

    git = get_git_cmd(wpt_root)
    files = git("diff", "--name-only", "-z", revish).split("\0")
    assert not files[-1]
    files = set(files[:-1])

    if include_uncommitted:
        entries = git("status", "-z").split("\0")
        assert not entries[-1]
        entries = entries[:-1]
        for item in entries:
            status, path = item.split()
            if status == "??" and not include_new:
                continue
            else:
                if not os.path.isdir(path):
                    files.add(path)
                else:
                    for dirpath, dirnames, filenames in os.walk(path):
                        for filename in filenames:
                            files.add(os.path.join(dirpath, filename))

    if not files:
        return [], []

    changed = []
    ignored = []
    for item in sorted(files):
        fullpath = os.path.join(wpt_root, item)
        topmost_dir = item.split(os.sep, 1)[0]
        if topmost_dir in ignore_dirs:
            ignored.append(fullpath)
        else:
            changed.append(fullpath)

    return changed, ignored


def _in_repo_root(full_path):
    rel_path = os.path.relpath(full_path, wpt_root)
    path_components = rel_path.split(os.sep)
    return len(path_components) < 2

def _init_manifest_cache():
    c = {}

    def load(manifest_path=None):
        if manifest_path is None:
            manifest_path = os.path.join(wpt_root, "MANIFEST.json")
        if c.get(manifest_path):
            return c[manifest_path]
        # cache at most one path:manifest
        c.clear()
        wpt_manifest = manifest.load(wpt_root, manifest_path)
        if wpt_manifest is None:
            wpt_manifest = manifest.Manifest()
        update.update(wpt_root, wpt_manifest)
        c[manifest_path] = wpt_manifest
        return c[manifest_path]
    return load

load_manifest = _init_manifest_cache()


def affected_testfiles(files_changed, skip_tests, manifest_path=None):
    """Determine and return list of test files that reference changed files."""
    affected_testfiles = set()
    # Exclude files that are in the repo root, because
    # they are not part of any test.
    files_changed = [f for f in files_changed if not _in_repo_root(f)]
    nontests_changed = set(files_changed)
    wpt_manifest = load_manifest(manifest_path)

    test_types = ["testharness", "reftest", "wdspec"]
    support_files = {os.path.join(wpt_root, path)
                     for _, path, _ in wpt_manifest.itertypes("support")}
    wdspec_test_files = {os.path.join(wpt_root, path)
                         for _, path, _ in wpt_manifest.itertypes("wdspec")}
    test_files = {os.path.join(wpt_root, path)
                  for _, path, _ in wpt_manifest.itertypes(*test_types)}

    nontests_changed = nontests_changed.intersection(support_files)

    tests_changed = set(item for item in files_changed if item in test_files)

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
                    file_contents = file_contents.decode("utf-16be", "replace")
                elif file_contents.startswith("\xff\xfe"):
                    file_contents = file_contents.decode("utf-16le", "replace")
                else:
                    file_contents = file_contents.decode("utf8", "replace")
                for full_path, repo_path in nontest_changed_paths:
                    rel_path = os.path.relpath(full_path, root).replace(os.path.sep, "/")
                    if rel_path in file_contents or repo_path in file_contents:
                        affected_testfiles.add(test_full_path)
                        continue

    return tests_changed, affected_testfiles


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("revish", default=None, help="Commits to consider. Defaults to the commits on the current branch", nargs="?")
    parser.add_argument("--ignore-dirs", nargs="*", type=set, default=set(["resources"]),
                        help="Directories to exclude from the list of changes")
    parser.add_argument("--modified", action="store_true",
                        help="Include files under version control that have been modified or staged")
    parser.add_argument("--new", action="store_true",
                        help="Include files in the worktree that are not in version control")
    parser.add_argument("--show-type", action="store_true",
                        help="Print the test type along with each affected test")
    return parser


def get_parser_affected():
    parser = get_parser()
    parser.add_argument("--metadata",
                        dest="metadata_root",
                        action="store",
                        default=wpt_root,
                        help="Directory that will contain MANIFEST.json")
    return parser

def get_revish(**kwargs):
    revish = kwargs["revish"]
    if kwargs["revish"] is None:
        revish = "%s..HEAD" % branch_point()
    return revish


def run_changed_files(**kwargs):
    revish = get_revish(**kwargs)
    changed, _ = files_changed(revish, kwargs["ignore_dirs"],
                               include_uncommitted=kwargs["modified"],
                               include_new=kwargs["new"])
    for item in sorted(changed):
        print(os.path.relpath(item, wpt_root))


def run_tests_affected(**kwargs):
    revish = get_revish(**kwargs)
    changed, _ = files_changed(revish, kwargs["ignore_dirs"],
                               include_uncommitted=kwargs["modified"],
                               include_new=kwargs["new"])
    manifest_path = os.path.join(kwargs["metadata_root"], "MANIFEST.json")
    tests_changed, dependents = affected_testfiles(
        changed,
        set(["conformance-checkers", "docs", "tools"]),
        manifest_path=manifest_path
    )

    message = "{path}"
    if kwargs["show_type"]:
        wpt_manifest = load_manifest(manifest_path)
        message = "{path}\t{item_type}"
    for item in sorted(tests_changed | dependents):
        results = {
            "path": os.path.relpath(item, wpt_root)
        }
        if kwargs["show_type"]:
            item_types = {i.item_type for i in wpt_manifest.iterpath(results["path"])}
            if len(item_types) != 1:
                item_types = [" ".join(item_types)]
            results["item_type"] = item_types.pop()
        print(message.format(**results))
