from __future__ import print_function
import os.path
import sys

import pkg_resources
import pytest

from .tree_construction import TreeConstructionFile
from .tokenizer import TokenizerFile
from .sanitizer import SanitizerFile

_dir = os.path.abspath(os.path.dirname(__file__))
_root = os.path.join(_dir, "..", "..")
_testdata = os.path.join(_dir, "testdata")
_tree_construction = os.path.join(_testdata, "tree-construction")
_tokenizer = os.path.join(_testdata, "tokenizer")
_sanitizer_testdata = os.path.join(_dir, "sanitizer-testdata")


def fail_if_missing_pytest_expect():
    """Throws an exception halting pytest if pytest-expect isn't working"""
    try:
        from pytest_expect import expect  # noqa
    except ImportError:
        header = '*' * 78
        print(
            '\n' +
            header + '\n' +
            'ERROR: Either pytest-expect or its dependency u-msgpack-python is not\n' +
            'installed. Please install them both before running pytest.\n' +
            header + '\n',
            file=sys.stderr
        )
        raise


fail_if_missing_pytest_expect()


def pytest_configure(config):
    msgs = []

    if not os.path.exists(_testdata):
        msg = "testdata not available! "
        if os.path.exists(os.path.join(_root, ".git")):
            msg += ("Please run git submodule update --init --recursive " +
                    "and then run tests again.")
        else:
            msg += ("The testdata doesn't appear to be included with this package, " +
                    "so finding the right version will be hard. :(")
        msgs.append(msg)

    if config.option.update_xfail:
        # Check for optional requirements
        req_file = os.path.join(_root, "requirements-optional.txt")
        if os.path.exists(req_file):
            with open(req_file, "r") as fp:
                for line in fp:
                    if (line.strip() and
                        not (line.startswith("-r") or
                             line.startswith("#"))):
                        if ";" in line:
                            spec, marker = line.strip().split(";", 1)
                        else:
                            spec, marker = line.strip(), None
                        req = pkg_resources.Requirement.parse(spec)
                        if marker and not pkg_resources.evaluate_marker(marker):
                            msgs.append("%s not available in this environment" % spec)
                        else:
                            try:
                                installed = pkg_resources.working_set.find(req)
                            except pkg_resources.VersionConflict:
                                msgs.append("Outdated version of %s installed, need %s" % (req.name, spec))
                            else:
                                if not installed:
                                    msgs.append("Need %s" % spec)

        # Check cElementTree
        import xml.etree.ElementTree as ElementTree

        try:
            import xml.etree.cElementTree as cElementTree
        except ImportError:
            msgs.append("cElementTree unable to be imported")
        else:
            if cElementTree.Element is ElementTree.Element:
                msgs.append("cElementTree is just an alias for ElementTree")

    if msgs:
        pytest.exit("\n".join(msgs))


def pytest_collect_file(path, parent):
    dir = os.path.abspath(path.dirname)
    dir_and_parents = set()
    while dir not in dir_and_parents:
        dir_and_parents.add(dir)
        dir = os.path.dirname(dir)

    if _tree_construction in dir_and_parents:
        if path.ext == ".dat":
            return TreeConstructionFile(path, parent)
    elif _tokenizer in dir_and_parents:
        if path.ext == ".test":
            return TokenizerFile(path, parent)
    elif _sanitizer_testdata in dir_and_parents:
        if path.ext == ".dat":
            return SanitizerFile(path, parent)
