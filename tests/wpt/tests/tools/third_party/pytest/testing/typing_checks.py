"""File for checking typing issues.

This file is not executed, it is only checked by mypy to ensure that
none of the code triggers any mypy errors.
"""
import pytest


# Issue #7488.
@pytest.mark.xfail(raises=RuntimeError)
def check_mark_xfail_raises() -> None:
    pass


# Issue #7494.
@pytest.fixture(params=[(0, 0), (1, 1)], ids=lambda x: str(x[0]))
def check_fixture_ids_callable() -> None:
    pass


# Issue #7494.
@pytest.mark.parametrize("func", [str, int], ids=lambda x: str(x.__name__))
def check_parametrize_ids_callable(func) -> None:
    pass
