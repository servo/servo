import pytest

from ..gitignore import fnmatch_translate, PathFilter

match_data = [
    ("foo", True, ["a/foo", "foo"]),
    ("*.a", True, ["foo.a", "a/foo.a", "a/b/foo.a", "a.a/foo.a"]),
    ("*.py[co]", True, ["a.pyc", "a.pyo", "a/b/c.pyc"]),
    ("\\#*", True, ["#a", "a/#b"]),
    ("*#", True, ["a#", "a/b#", "#a#"]),
    ("/*.c", True, ["a.c", ".c"]),
    ("**/b", False, ["a/b", "a/c/b"]),
    ("*b", True, ["ab"]),
    ("*b", True, ["a/b"]),
    ("**/b", False, ["a/b"]),
    ("a/", True, ["a"]),
    ("a[/]b", True, []),
    ("**/b", False, ["a/c/b"]),
    ("a?c", True, ["abc"]),
    ("a[^b]c", True, ["acc"]),
    ("a[b-c]c", True, ["abc", "acc"]),
    ("a[^]c", True, ["ac"]),  # This is probably wrong
    ("a[^]c", True, ["ac"]),  # This is probably wrong
]

mismatch_data = [
    ("foo", True, ["foob", "afoo"]),
    ("*.a", True, ["a", "foo:a", "a.a/foo"]),
    ("*.py[co]", True, ["a.pyd", "pyo", "a.py"]),
    ("a", True, ["ab"]),
    ("a?c", True, ["ac", "abbc"]),
    ("a[^b]c", True, ["abc"]),
    ("a[b-c]c", True, ["adc"]),
]

invalid_data = [
    "[a",
    "***/foo",
    "a\\",
    "**b",
    "b**/",
    "[[]"
]

filter_data = [
    (["foo", "bar/", "/a", "*.py"],
     [("", ["foo", "bar", "baz"], ["a"]),
      ("baz", ["a"], ["foo", "bar"])],
     [(["baz"], []),
      (["a"], ["bar"])]),
    (["#foo", "", "a*", "!a.py"],
     [("", ["foo"], ["a", "a.foo", "a.py"])],
     [(["foo"], ["a.py"])]),
]


def expand_data(compact_data):
    for pattern, name_only, inputs in compact_data:
        for input in inputs:
            yield pattern, name_only, input


@pytest.mark.parametrize("pattern, name_only, input", expand_data(match_data))
def tests_match(pattern, name_only, input):
    name_only_result, regexp = fnmatch_translate(pattern)
    assert name_only_result == name_only
    if name_only:
        input = input.rsplit("/", 1)[-1]
    assert regexp.match(input) is not None


@pytest.mark.parametrize("pattern, name_only, input", expand_data(mismatch_data))
def tests_no_match(pattern, name_only, input):
    name_only_result, regexp = fnmatch_translate(pattern)
    assert name_only_result == name_only
    if name_only:
        input = input.rsplit("/", 1)[-1]
    assert regexp.match(input) is None


@pytest.mark.parametrize("pattern", invalid_data)
def tests_invalid(pattern):
    with pytest.raises(ValueError):
        fnmatch_translate(pattern)


@pytest.mark.parametrize("rules, input, expected", filter_data)
def test_path_filter(rules, input, expected):
    f = PathFilter(None, rules)
    # Add some fake stat data
    for i, item in enumerate(input):
        repl = [input[i][0]]
        for j in [1, 2]:
            repl.append([(name, None) for name in input[i][j]])
        input[i] = tuple(repl)

    for i, output in enumerate(f(input)):
        assert output[0] == input[i][0]
        for j in [1, 2]:
            assert [item[0] for item in output[j]] == expected[i][j-1]
