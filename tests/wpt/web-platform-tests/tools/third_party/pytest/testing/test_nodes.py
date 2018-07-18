import pytest

from _pytest import nodes


@pytest.mark.parametrize(
    "baseid, nodeid, expected",
    (
        ("", "", True),
        ("", "foo", True),
        ("", "foo/bar", True),
        ("", "foo/bar::TestBaz::()", True),
        ("foo", "food", False),
        ("foo/bar::TestBaz::()", "foo/bar", False),
        ("foo/bar::TestBaz::()", "foo/bar::TestBop::()", False),
        ("foo/bar", "foo/bar::TestBop::()", True),
    ),
)
def test_ischildnode(baseid, nodeid, expected):
    result = nodes.ischildnode(baseid, nodeid)
    assert result is expected
