import pytest

from ..gitignore import fnmatch_translate, PathFilter

match_data = [
    ("foo", False, ["a/foo", "foo"]),
    ("*.a", False, ["foo.a", "a/foo.a", "a/b/foo.a", "a.a/foo.a"]),
    ("*.py[co]", False, ["a.pyc", "a.pyo", "a/b/c.pyc"]),
    ("\\#*", False, ["#a", "a/#b"]),
    ("*#", False, ["a#", "a/b#", "#a#"]),
    ("/*.c", False, ["a.c", ".c"]),
    ("**/b", False, ["a/b", "a/c/b"]),
    ("*b", True, ["ab"]),
    ("**/b", True, ["a/b"]),
    ("a/", True, ["a", "a/b", "a/b/c"])
]

mismatch_data = [
    ("foo", False, ["foob", "afoo"]),
    ("*.a", False, ["a", "foo:a", "a.a/foo"]),
    ("*.py[co]", False, ["a.pyd", "pyo"]),
    ("/*.c", False, ["a/b.c"]),
    ("*b", True, ["a/b"]),
    ("**b", True, ["a/b"]),
    ("a[/]b", True, ["a/b"]),
    ("**/b", True, ["a/c/b"]),
    ("a", True, ["ab"])
]

invalid_data = [
    "[a",
    "***/foo",
    "a\\",
]

filter_data = [
    ("foo", True),
    ("a", False),
    ("a/b", False),
    ("a/c", True),
    ("a/c/", False),
    ("c/b", True)
]


def expand_data(compact_data):
    for pattern, path_name, inputs in compact_data:
        for input in inputs:
            yield pattern, input, path_name


@pytest.mark.parametrize("pattern, input, path_name", expand_data(match_data))
def tests_match(pattern, input, path_name):
    regexp = fnmatch_translate(pattern, path_name)
    assert regexp.match(input) is not None


@pytest.mark.parametrize("pattern, input, path_name", expand_data(mismatch_data))
def tests_no_match(pattern, input, path_name):
    regexp = fnmatch_translate(pattern, path_name)
    assert regexp.match(input) is None


@pytest.mark.parametrize("pattern", invalid_data)
def tests_invalid(pattern):
    with pytest.raises(ValueError):
        fnmatch_translate(pattern, False)
    with pytest.raises(ValueError):
        fnmatch_translate(pattern, True)


@pytest.mark.parametrize("path, expected", filter_data)
def test_path_filter(path, expected):
    extras = [
        "#foo",
        "a  ",
        "**/b",
        "a/c/",
        "!c/b",
    ]
    f = PathFilter(None, extras)
    assert f(path) == expected
