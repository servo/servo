import argparse
import os
import re
from ..wpt.testfiles import branch_point, files_changed

from tools import localpaths  # noqa: F401
from six import iteritems

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))

# Common exclusions between affected_tests and stability jobs.
# Files in these dirs would trigger the execution of too many tests.
EXCLUDES = [
    "!tools/",
    "!docs/",
    "!conformance-checkers/",
    "!.*/OWNERS",
    "!.*/META.yml",
    "!.*/tools/",
    "!.*/README",
    "!css/[^/]*$"
]

# Rules are just regex on the path, with a leading ! indicating a regex that must not
# match for the job
job_path_map = {
    "affected_tests": [".*/.*", "!resources/(?!idlharness.js)"] + EXCLUDES,
    "stability": [".*/.*", "!resources/.*"] + EXCLUDES,
    "lint": [".*"],
    "manifest_upload": [".*"],
    "resources_unittest": ["resources/", "tools/"],
    "tools_unittest": ["tools/"],
    "wptrunner_unittest": ["tools/"],
    "build_css": ["css/"],
    "update_built": ["update-built-tests\\.sh",
                     "2dcontext/",
                     "infrastructure/",
                     "html/",
                     "offscreen-canvas/",
                     "mimesniff/",
                     "css/css-ui/",
                     "WebIDL"],
    "wpt_integration": ["tools/"],
    "wptrunner_infrastructure": ["infrastructure/",
                                 "tools/",
                                 "resources/",
                                 "webdriver/tests/support"],
}


def _path_norm(path):
    """normalize a path for both case and slashes (to /)"""
    path = os.path.normcase(path)
    if os.path.sep != "/":
        # this must be after the normcase call as that does slash normalization
        path = path.replace(os.path.sep, "/")
    return path


class Ruleset(object):
    def __init__(self, rules):
        self.include = []
        self.exclude = []
        for rule in rules:
            rule = _path_norm(rule)
            self.add_rule(rule)

    def add_rule(self, rule):
        if rule.startswith("!"):
            target = self.exclude
            rule = rule[1:]
        else:
            target = self.include

        target.append(re.compile("^%s" % rule))

    def __call__(self, path):
        path = _path_norm(path)
        for item in self.exclude:
            if item.match(path):
                return False
        for item in self.include:
            if item.match(path):
                return True
        return False

    def __repr__(self):
        subs = tuple(",".join(item.pattern for item in target)
                     for target in (self.include, self.exclude))
        return "Rules<include:[%s] exclude:[%s]>" % subs


def get_paths(**kwargs):
    if kwargs["revish"] is None:
        revish = "%s..HEAD" % branch_point()
    else:
        revish = kwargs["revish"]

    changed, _ = files_changed(revish)
    all_changed = {os.path.relpath(item, wpt_root) for item in set(changed)}
    return all_changed


def get_jobs(paths, **kwargs):
    if kwargs.get("all"):
        return set(job_path_map.keys())

    jobs = set()

    rules = {}
    includes = kwargs.get("includes")
    if includes is not None:
        includes = set(includes)
    for key, value in iteritems(job_path_map):
        if includes is None or key in includes:
            rules[key] = Ruleset(value)

    for path in paths:
        for job in list(rules.keys()):
            ruleset = rules[job]
            if ruleset(path):
                rules.pop(job)
                jobs.add(job)
        if not rules:
            break

    # Default jobs shuld run even if there were no changes
    if not paths:
        for job, path_re in iteritems(job_path_map):
            if ".*" in path_re:
                jobs.add(job)

    return jobs


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("revish", default=None, help="Commits to consider. Defaults to the commits on the current branch", nargs="?")
    parser.add_argument("--all", help="List all jobs unconditionally.", action="store_true")
    parser.add_argument("--includes", default=None, help="Jobs to check for. Return code is 0 if all jobs are found, otherwise 1", nargs="*")
    return parser


def run(**kwargs):
    paths = get_paths(**kwargs)
    jobs = get_jobs(paths, **kwargs)
    if not kwargs["includes"]:
        for item in sorted(jobs):
            print(item)
    else:
        return 0 if set(kwargs["includes"]).issubset(jobs) else 1
