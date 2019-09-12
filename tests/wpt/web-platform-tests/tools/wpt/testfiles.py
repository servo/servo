import argparse
import logging
import os
import re
import subprocess
import sys

import six
from collections import OrderedDict
from six import iteritems

try:
    from ..manifest import manifest
except ValueError:
    # if we're not within the tools package, the above is an import from above
    # the top-level which raises ValueError, so reimport it with an absolute
    # reference
    #
    # note we need both because depending on caller we may/may not have the
    # paths set up correctly to handle both and MYPY has no knowledge of our
    # sys.path magic
    from manifest import manifest  # type: ignore

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Callable
    from typing import Dict
    from typing import Iterable
    from typing import List
    from typing import Optional
    from typing import Pattern
    from typing import Sequence
    from typing import Set
    from typing import Text
    from typing import Tuple
    from typing import Union

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

logger = logging.getLogger()


def get_git_cmd(repo_path):
    # type: (bytes) -> Callable[..., Text]
    """Create a function for invoking git commands as a subprocess."""
    def git(cmd, *args):
        # type: (Text, *Union[bytes, Text]) -> Text
        full_cmd = [u"git", cmd] + list(item.decode("utf8") if isinstance(item, bytes) else item for item in args)  # type: List[Text]
        try:
            logger.debug(" ".join(full_cmd))
            return subprocess.check_output(full_cmd, cwd=repo_path).decode("utf8").strip()
        except subprocess.CalledProcessError as e:
            logger.error("Git command exited with status %i" % e.returncode)
            logger.error(e.output)
            sys.exit(1)
    return git


def display_branch_point():
    # type: () -> None
    print(branch_point())


def branch_point():
    # type: () -> Optional[Text]
    git = get_git_cmd(wpt_root)
    if (os.environ.get("GITHUB_PULL_REQUEST", "false") == "false" and
        os.environ.get("GITHUB_BRANCH") == "master"):
        # For builds on the master branch just return the HEAD commit
        return git("rev-parse", "HEAD")
    elif os.environ.get("GITHUB_PULL_REQUEST", "false") != "false":
        # This is a PR, so the base branch is in GITHUB_BRANCH
        base_branch = os.environ.get("GITHUB_BRANCH")
        assert base_branch, "GITHUB_BRANCH environment variable is defined"
        branch_point = git("merge-base", "HEAD", base_branch)  # type: Optional[Text]
    else:
        # Otherwise we aren't on a PR, so we try to find commits that are only in the
        # current branch c.f.
        # http://stackoverflow.com/questions/13460152/find-first-ancestor-commit-in-another-branch

        # parse HEAD into an object ref
        head = git("rev-parse", "HEAD")

        # get everything in refs/heads and refs/remotes that doesn't include HEAD
        not_heads = [item for item in git("rev-parse", "--not", "--branches", "--remotes").split("\n")
                     if item != "^%s" % head]

        # get all commits on HEAD but not reachable from anything in not_heads
        commits = git("rev-list", "--topo-order", "--parents", "HEAD", *not_heads)
        commit_parents = OrderedDict()  # type: Dict[Text, List[Text]]
        if commits:
            for line in commits.split("\n"):
                line_commits = line.split(" ")
                commit_parents[line_commits[0]] = line_commits[1:]

        branch_point = None

        # if there are any commits, take the first parent that is not in commits
        for commit, parents in iteritems(commit_parents):
            for parent in parents:
                if parent not in commit_parents:
                    branch_point = parent
                    break

            if branch_point:
                break

        # if we had any commits, we should now have a branch point
        assert branch_point or not commit_parents

        # The above heuristic will fail in the following cases:
        #
        # - The current branch has fallen behind the remote version
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


def compile_ignore_rule(rule):
    # type: (str) -> Pattern[str]
    rule = rule.replace(os.path.sep, "/")
    parts = rule.split("/")
    re_parts = []
    for part in parts:
        if part.endswith("**"):
            re_parts.append(re.escape(part[:-2]) + ".*")
        elif part.endswith("*"):
            re_parts.append(re.escape(part[:-1]) + "[^/]*")
        else:
            re_parts.append(re.escape(part))
    return re.compile("^%s$" % "/".join(re_parts))


def repo_files_changed(revish, include_uncommitted=False, include_new=False):
    # type: (str, bool, bool) -> Set[Text]
    git = get_git_cmd(wpt_root)
    files_list = git("diff", "--name-only", "-z", revish).split("\0")
    assert not files_list[-1]
    files = set(files_list[:-1])

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

    return files


def exclude_ignored(files, ignore_rules):
    # type: (Iterable[Text], Optional[Sequence[str]]) -> Tuple[List[Text], List[Text]]
    if ignore_rules is None:
        ignore_rules = []
    compiled_ignore_rules = [compile_ignore_rule(item) for item in ignore_rules]

    changed = []
    ignored = []
    for item in sorted(files):
        fullpath = os.path.join(wpt_root, item)
        rule_path = item.replace(os.path.sep, "/")
        for rule in compiled_ignore_rules:
            if rule.match(rule_path):
                ignored.append(fullpath)
                break
        else:
            changed.append(fullpath)

    return changed, ignored


def files_changed(revish,  # type: str
                  ignore_rules=None,  # type: Optional[Sequence[str]]
                  include_uncommitted=False,  # type: bool
                  include_new=False  # type: bool
                  ):
    # type: (...) -> Tuple[List[Text], List[Text]]
    """Find files changed in certain revisions.

    The function passes `revish` directly to `git diff`, so `revish` can have a
    variety of forms; see `git diff --help` for details. Files in the diff that
    are matched by `ignore_rules` are excluded.
    """
    files = repo_files_changed(revish,
                               include_uncommitted=include_uncommitted,
                               include_new=include_new)
    if not files:
        return [], []

    return exclude_ignored(files, ignore_rules)


def _in_repo_root(full_path):
    # type: (Union[bytes, Text]) -> bool
    rel_path = os.path.relpath(full_path, wpt_root)
    path_components = rel_path.split(os.sep)
    return len(path_components) < 2


def load_manifest(manifest_path=None, manifest_update=True):
    # type: (Optional[str], bool) -> manifest.Manifest
    if manifest_path is None:
        manifest_path = os.path.join(wpt_root, "MANIFEST.json")
    return manifest.load_and_update(wpt_root, manifest_path, "/",
                                    update=manifest_update)


def affected_testfiles(files_changed,  # type: Iterable[Text]
                       skip_dirs=None,  # type: Optional[Set[str]]
                       manifest_path=None,  # type: Optional[str]
                       manifest_update=True  # type: bool
                       ):
    # type: (...) -> Tuple[Set[Text], Set[str]]
    """Determine and return list of test files that reference changed files."""
    if skip_dirs is None:
        skip_dirs = {"conformance-checkers", "docs", "tools"}
    affected_testfiles = set()
    # Exclude files that are in the repo root, because
    # they are not part of any test.
    files_changed = [f for f in files_changed if not _in_repo_root(f)]
    nontests_changed = set(files_changed)
    wpt_manifest = load_manifest(manifest_path, manifest_update)

    test_types = ["testharness", "reftest", "wdspec"]
    support_files = {os.path.join(wpt_root, path)
                     for _, path, _ in wpt_manifest.itertypes("support")}
    wdspec_test_files = {os.path.join(wpt_root, path)
                         for _, path, _ in wpt_manifest.itertypes("wdspec")}
    test_files = {os.path.join(wpt_root, path)
                  for _, path, _ in wpt_manifest.itertypes(*test_types)}

    interface_dir = os.path.join(wpt_root, 'interfaces')
    interfaces_files = {os.path.join(wpt_root, 'interfaces', filename)
                        for filename in os.listdir(interface_dir)}

    interfaces_changed = interfaces_files.intersection(nontests_changed)
    nontests_changed = nontests_changed.intersection(support_files)

    tests_changed = {item for item in files_changed if item in test_files}

    nontest_changed_paths = set()
    rewrites = {"/resources/webidl2/lib/webidl2.js": "/resources/WebIDLParser.js"}  # type: Dict[Text, Text]
    for full_path in nontests_changed:
        rel_path = os.path.relpath(full_path, wpt_root)
        path_components = rel_path.split(os.sep)
        top_level_subdir = path_components[0]
        if top_level_subdir in skip_dirs:
            continue
        repo_path = "/" + os.path.relpath(full_path, wpt_root).replace(os.path.sep, "/")
        if repo_path in rewrites:
            repo_path = rewrites[repo_path]
            full_path = os.path.join(wpt_root, repo_path[1:].replace("/", os.path.sep))
        nontest_changed_paths.add((full_path, repo_path))

    interface_name = lambda x: os.path.splitext(os.path.basename(x))[0]
    interfaces_changed_names = map(interface_name, interfaces_changed)

    def affected_by_wdspec(test):
        # type: (str) -> bool
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

    def affected_by_interfaces(file_contents):
        # type: (Union[bytes, Text]) -> bool
        if len(interfaces_changed_names) > 0:
            if 'idlharness.js' in file_contents:
                for interface in interfaces_changed_names:
                    regex = '[\'"]' + interface + '(\\.idl)?[\'"]'
                    if re.search(regex, file_contents):
                        return True
        return False

    for root, dirs, fnames in os.walk(wpt_root):
        # Walk top_level_subdir looking for test files containing either the
        # relative filepath or absolute filepath to the changed files.
        if root == wpt_root:
            for dir_name in skip_dirs:
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
                raw_file_contents = fh.read()  # type: bytes
                if raw_file_contents.startswith("\xfe\xff"):
                    file_contents = raw_file_contents.decode("utf-16be", "replace")  # type: Text
                elif raw_file_contents.startswith("\xff\xfe"):
                    file_contents = raw_file_contents.decode("utf-16le", "replace")
                else:
                    file_contents = raw_file_contents.decode("utf8", "replace")
                for full_path, repo_path in nontest_changed_paths:
                    rel_path = os.path.relpath(full_path, root).replace(os.path.sep, "/")
                    if rel_path in file_contents or repo_path in file_contents or affected_by_interfaces(file_contents):
                        affected_testfiles.add(test_full_path)
                        continue

    return tests_changed, affected_testfiles


def get_parser():
    # type: () -> argparse.ArgumentParser
    parser = argparse.ArgumentParser()
    parser.add_argument("revish", default=None, help="Commits to consider. Defaults to the "
                        "commits on the current branch", nargs="?")
    # TODO: Consolidate with `./wpt run --affected`:
    # https://github.com/web-platform-tests/wpt/issues/14560
    parser.add_argument("--ignore-rules", nargs="*", type=set,  # type: ignore
                        default={"resources/testharness*"},
                        help="Rules for paths to exclude from lists of changes. Rules are paths "
                        "relative to the test root, with * before a separator or the end matching "
                        "anything other than a path separator and ** in that position matching "
                        "anything")
    parser.add_argument("--modified", action="store_true",
                        help="Include files under version control that have been "
                        "modified or staged")
    parser.add_argument("--new", action="store_true",
                        help="Include files in the worktree that are not in version control")
    parser.add_argument("--show-type", action="store_true",
                        help="Print the test type along with each affected test")
    parser.add_argument("--null", action="store_true",
                        help="Separate items with a null byte")
    return parser


def get_parser_affected():
    # type: () -> argparse.ArgumentParser
    parser = get_parser()
    parser.add_argument("--metadata",
                        dest="metadata_root",
                        action="store",
                        default=wpt_root,
                        help="Directory that will contain MANIFEST.json")
    return parser


def get_revish(**kwargs):
    # type: (**Any) -> bytes
    revish = kwargs.get("revish")
    if revish is None:
        revish = "%s..HEAD" % branch_point()
    if isinstance(revish, six.text_type):
        revish = revish.encode("utf8")
    assert isinstance(revish, six.binary_type)
    return revish


def run_changed_files(**kwargs):
    # type: (**Any) -> None
    revish = get_revish(**kwargs)
    changed, _ = files_changed(revish, kwargs["ignore_rules"],
                               include_uncommitted=kwargs["modified"],
                               include_new=kwargs["new"])

    separator = "\0" if kwargs["null"] else "\n"

    for item in sorted(changed):
        sys.stdout.write(os.path.relpath(item.encode("utf8"), wpt_root) + separator)


def run_tests_affected(**kwargs):
    # type: (**Any) -> None
    revish = get_revish(**kwargs)
    changed, _ = files_changed(revish, kwargs["ignore_rules"],
                               include_uncommitted=kwargs["modified"],
                               include_new=kwargs["new"])
    manifest_path = os.path.join(kwargs["metadata_root"], "MANIFEST.json")
    tests_changed, dependents = affected_testfiles(
        changed,
        {"conformance-checkers", "docs", "tools"},
        manifest_path=manifest_path
    )

    message = "{path}"
    if kwargs["show_type"]:
        wpt_manifest = load_manifest(manifest_path)
        message = "{path}\t{item_type}"

    message += "\0" if kwargs["null"] else "\n"

    for item in sorted(tests_changed | dependents):
        results = {
            "path": os.path.relpath(item, wpt_root)
        }
        if kwargs["show_type"]:
            item_types = {i.item_type for i in wpt_manifest.iterpath(results["path"])}
            if len(item_types) != 1:
                item_types = {" ".join(item_types)}
            results["item_type"] = item_types.pop()
        sys.stdout.write(message.format(**results))
