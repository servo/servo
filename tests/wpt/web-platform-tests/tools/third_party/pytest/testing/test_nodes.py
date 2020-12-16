# -*- coding: utf-8 -*-
import py

import pytest
from _pytest import nodes


@pytest.mark.parametrize(
    "baseid, nodeid, expected",
    (
        ("", "", True),
        ("", "foo", True),
        ("", "foo/bar", True),
        ("", "foo/bar::TestBaz", True),
        ("foo", "food", False),
        ("foo/bar::TestBaz", "foo/bar", False),
        ("foo/bar::TestBaz", "foo/bar::TestBop", False),
        ("foo/bar", "foo/bar::TestBop", True),
    ),
)
def test_ischildnode(baseid, nodeid, expected):
    result = nodes.ischildnode(baseid, nodeid)
    assert result is expected


def test_std_warn_not_pytestwarning(testdir):
    items = testdir.getitems(
        """
        def test():
            pass
    """
    )
    with pytest.raises(ValueError, match=".*instance of PytestWarning.*"):
        items[0].warn(UserWarning("some warning"))


def test__check_initialpaths_for_relpath():
    """Ensure that it handles dirs, and does not always use dirname."""
    cwd = py.path.local()

    class FakeSession:
        _initialpaths = [cwd]

    assert nodes._check_initialpaths_for_relpath(FakeSession, cwd) == ""

    sub = cwd.join("file")

    class FakeSession:
        _initialpaths = [cwd]

    assert nodes._check_initialpaths_for_relpath(FakeSession, sub) == "file"

    outside = py.path.local("/outside")
    assert nodes._check_initialpaths_for_relpath(FakeSession, outside) is None
