# mypy: allow-untyped-defs

import pytest

from ..gitignore import fnmatch_translate, PathFilter

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Tuple
    from typing import Iterable
    from typing import Sequence

match_data = [
    (b"foo", True, [b"a/foo", b"foo"]),
    (b"*.a", True, [b"foo.a", b"a/foo.a", b"a/b/foo.a", b"a.a/foo.a"]),
    (b"*.py[co]", True, [b"a.pyc", b"a.pyo", b"a/b/c.pyc"]),
    (b"\\#*", True, [b"#a", b"a/#b"]),
    (b"*#", True, [b"a#", b"a/b#", b"#a#"]),
    (b"/*.c", True, [b"a.c", b".c"]),
    (b"**/b", False, [b"a/b", b"a/c/b"]),
    (b"*b", True, [b"ab"]),
    (b"*b", True, [b"a/b"]),
    (b"**/b", False, [b"a/b"]),
    (b"a/", True, [b"a"]),
    (b"a[/]b", True, []),
    (b"**/b", False, [b"a/c/b"]),
    (b"a?c", True, [b"abc"]),
    (b"a[^b]c", True, [b"acc"]),
    (b"a[b-c]c", True, [b"abc", b"acc"]),
    (b"a[]c", True, [b"ac"]),
]  # type: Sequence[Tuple[bytes, bool, Iterable[bytes]]]

mismatch_data = [
    (b"foo", True, [b"foob", b"afoo"]),
    (b"*.a", True, [b"a", b"foo:a", b"a.a/foo"]),
    (b"*.py[co]", True, [b"a.pyd", b"pyo", b"a.py"]),
    (b"a", True, [b"ab"]),
    (b"a?c", True, [b"ac", b"abbc"]),
    (b"a[^b]c", True, [b"abc"]),
    (b"a[b-c]c", True, [b"adc"]),
]  # type: Sequence[Tuple[bytes, bool, Iterable[bytes]]]

invalid_data = [
    b"[a",
    b"***/foo",
    b"a\\",
    b"**b",
    b"b**/",
    b"[[]",
    b"a[^]c",
]

filter_data = [
    ([b"foo", b"bar/", b"/a", b"*.py"],
     [(b"", [b"foo", b"bar", b"baz"], [b"a"]),
      (b"baz", [b"a"], [b"foo", b"bar"])],
     [([b"baz"], []),
      ([b"a"], [b"bar"])]),
    ([b"#foo", b"", b"a*", b"!a.py"],
     [(b"", [b"foo"], [b"a", b"a.foo", b"a.py"])],
     [([b"foo"], [b"a.py"])]),
    ([b"a.foo", b"!a.py"],
     [(b"", [b"foo"], [b"a", b"a.foo", b"a.py"])],
     [([b"foo"], [b"a", b"a.py"])]),
]


def expand_data(compact_data):
    # type: (Sequence[Tuple[bytes, bool, Iterable[bytes]]]) -> Iterable[Tuple[bytes, bool, bytes]]
    for pattern, name_only, inputs in compact_data:
        for input in inputs:
            yield pattern, name_only, input


@pytest.mark.parametrize("pattern, name_only, input", expand_data(match_data))
def tests_match(pattern, name_only, input):
    name_only_result, regexp = fnmatch_translate(pattern)
    assert name_only_result == name_only
    if name_only:
        input = input.rsplit(b"/", 1)[-1]
    assert regexp.match(input) is not None


@pytest.mark.parametrize("pattern, name_only, input", expand_data(mismatch_data))
def tests_no_match(pattern, name_only, input):
    name_only_result, regexp = fnmatch_translate(pattern)
    assert name_only_result == name_only
    if name_only:
        input = input.rsplit(b"/", 1)[-1]
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
