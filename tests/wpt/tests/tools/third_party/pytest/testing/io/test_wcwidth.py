from _pytest._io.wcwidth import wcswidth
from _pytest._io.wcwidth import wcwidth
import pytest


@pytest.mark.parametrize(
    ("c", "expected"),
    [
        ("\0", 0),
        ("\n", -1),
        ("a", 1),
        ("1", 1),
        ("א", 1),
        ("\u200b", 0),
        ("\u1abe", 0),
        ("\u0591", 0),
        ("🉐", 2),
        ("＄", 2),  # noqa: RUF001
    ],
)
def test_wcwidth(c: str, expected: int) -> None:
    assert wcwidth(c) == expected


@pytest.mark.parametrize(
    ("s", "expected"),
    [
        ("", 0),
        ("hello, world!", 13),
        ("hello, world!\n", -1),
        ("0123456789", 10),
        ("שלום, עולם!", 11),
        ("שְבֻעָיים", 6),
        ("🉐🉐🉐", 6),
    ],
)
def test_wcswidth(s: str, expected: int) -> None:
    assert wcswidth(s) == expected
