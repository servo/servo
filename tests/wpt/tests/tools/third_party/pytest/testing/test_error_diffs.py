"""
Tests and examples for correct "+/-" usage in error diffs.

See https://github.com/pytest-dev/pytest/issues/3333 for details.

"""
import sys

import pytest
from _pytest.pytester import Pytester


TESTCASES = [
    pytest.param(
        """
        def test_this():
            result =   [1, 4, 3]
            expected = [1, 2, 3]
            assert result == expected
        """,
        """
        >       assert result == expected
        E       assert [1, 4, 3] == [1, 2, 3]
        E         At index 1 diff: 4 != 2
        E         Full diff:
        E         - [1, 2, 3]
        E         ?     ^
        E         + [1, 4, 3]
        E         ?     ^
        """,
        id="Compare lists, one item differs",
    ),
    pytest.param(
        """
        def test_this():
            result =   [1, 2, 3]
            expected = [1, 2]
            assert result == expected
        """,
        """
        >       assert result == expected
        E       assert [1, 2, 3] == [1, 2]
        E         Left contains one more item: 3
        E         Full diff:
        E         - [1, 2]
        E         + [1, 2, 3]
        E         ?      +++
        """,
        id="Compare lists, one extra item",
    ),
    pytest.param(
        """
        def test_this():
            result =   [1, 3]
            expected = [1, 2, 3]
            assert result == expected
        """,
        """
        >       assert result == expected
        E       assert [1, 3] == [1, 2, 3]
        E         At index 1 diff: 3 != 2
        E         Right contains one more item: 3
        E         Full diff:
        E         - [1, 2, 3]
        E         ?     ---
        E         + [1, 3]
        """,
        id="Compare lists, one item missing",
    ),
    pytest.param(
        """
        def test_this():
            result =   (1, 4, 3)
            expected = (1, 2, 3)
            assert result == expected
        """,
        """
        >       assert result == expected
        E       assert (1, 4, 3) == (1, 2, 3)
        E         At index 1 diff: 4 != 2
        E         Full diff:
        E         - (1, 2, 3)
        E         ?     ^
        E         + (1, 4, 3)
        E         ?     ^
        """,
        id="Compare tuples",
    ),
    pytest.param(
        """
        def test_this():
            result =   {1, 3, 4}
            expected = {1, 2, 3}
            assert result == expected
        """,
        """
        >       assert result == expected
        E       assert {1, 3, 4} == {1, 2, 3}
        E         Extra items in the left set:
        E         4
        E         Extra items in the right set:
        E         2
        E         Full diff:
        E         - {1, 2, 3}
        E         ?     ^  ^
        E         + {1, 3, 4}
        E         ?     ^  ^
        """,
        id="Compare sets",
    ),
    pytest.param(
        """
        def test_this():
            result =   {1: 'spam', 3: 'eggs'}
            expected = {1: 'spam', 2: 'eggs'}
            assert result == expected
        """,
        """
        >       assert result == expected
        E       AssertionError: assert {1: 'spam', 3: 'eggs'} == {1: 'spam', 2: 'eggs'}
        E         Common items:
        E         {1: 'spam'}
        E         Left contains 1 more item:
        E         {3: 'eggs'}
        E         Right contains 1 more item:
        E         {2: 'eggs'}
        E         Full diff:
        E         - {1: 'spam', 2: 'eggs'}
        E         ?             ^
        E         + {1: 'spam', 3: 'eggs'}
        E         ?             ^
        """,
        id="Compare dicts with differing keys",
    ),
    pytest.param(
        """
        def test_this():
            result =   {1: 'spam', 2: 'eggs'}
            expected = {1: 'spam', 2: 'bacon'}
            assert result == expected
        """,
        """
        >       assert result == expected
        E       AssertionError: assert {1: 'spam', 2: 'eggs'} == {1: 'spam', 2: 'bacon'}
        E         Common items:
        E         {1: 'spam'}
        E         Differing items:
        E         {2: 'eggs'} != {2: 'bacon'}
        E         Full diff:
        E         - {1: 'spam', 2: 'bacon'}
        E         ?                 ^^^^^
        E         + {1: 'spam', 2: 'eggs'}
        E         ?                 ^^^^
        """,
        id="Compare dicts with differing values",
    ),
    pytest.param(
        """
        def test_this():
            result =   {1: 'spam', 2: 'eggs'}
            expected = {1: 'spam', 3: 'bacon'}
            assert result == expected
        """,
        """
        >       assert result == expected
        E       AssertionError: assert {1: 'spam', 2: 'eggs'} == {1: 'spam', 3: 'bacon'}
        E         Common items:
        E         {1: 'spam'}
        E         Left contains 1 more item:
        E         {2: 'eggs'}
        E         Right contains 1 more item:
        E         {3: 'bacon'}
        E         Full diff:
        E         - {1: 'spam', 3: 'bacon'}
        E         ?             ^   ^^^^^
        E         + {1: 'spam', 2: 'eggs'}
        E         ?             ^   ^^^^
        """,
        id="Compare dicts with differing items",
    ),
    pytest.param(
        """
        def test_this():
            result =   "spmaeggs"
            expected = "spameggs"
            assert result == expected
        """,
        """
        >       assert result == expected
        E       AssertionError: assert 'spmaeggs' == 'spameggs'
        E         - spameggs
        E         ?    -
        E         + spmaeggs
        E         ?   +
        """,
        id="Compare strings",
    ),
    pytest.param(
        """
        def test_this():
            result =   "spam bacon eggs"
            assert "bacon" not in result
        """,
        """
        >       assert "bacon" not in result
        E       AssertionError: assert 'bacon' not in 'spam bacon eggs'
        E         'bacon' is contained here:
        E           spam bacon eggs
        E         ?      +++++
        """,
        id='Test "not in" string',
    ),
]
if sys.version_info[:2] >= (3, 7):
    TESTCASES.extend(
        [
            pytest.param(
                """
                from dataclasses import dataclass

                @dataclass
                class A:
                    a: int
                    b: str

                def test_this():
                    result =   A(1, 'spam')
                    expected = A(2, 'spam')
                    assert result == expected
                """,
                """
                >       assert result == expected
                E       AssertionError: assert A(a=1, b='spam') == A(a=2, b='spam')
                E         Matching attributes:
                E         ['b']
                E         Differing attributes:
                E         ['a']
                E         Drill down into differing attribute a:
                E           a: 1 != 2
                E           +1
                E           -2
                """,
                id="Compare data classes",
            ),
            pytest.param(
                """
                import attr

                @attr.s(auto_attribs=True)
                class A:
                    a: int
                    b: str

                def test_this():
                    result =   A(1, 'spam')
                    expected = A(1, 'eggs')
                    assert result == expected
                """,
                """
                >       assert result == expected
                E       AssertionError: assert A(a=1, b='spam') == A(a=1, b='eggs')
                E         Matching attributes:
                E         ['a']
                E         Differing attributes:
                E         ['b']
                E         Drill down into differing attribute b:
                E           b: 'spam' != 'eggs'
                E           - eggs
                E           + spam
                """,
                id="Compare attrs classes",
            ),
        ]
    )


@pytest.mark.parametrize("code, expected", TESTCASES)
def test_error_diff(code: str, expected: str, pytester: Pytester) -> None:
    expected_lines = [line.lstrip() for line in expected.splitlines()]
    p = pytester.makepyfile(code)
    result = pytester.runpytest(p, "-vv")
    result.stdout.fnmatch_lines(expected_lines)
    assert result.ret == 1
