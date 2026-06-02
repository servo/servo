"""Test importing of all internal packages and modules.

This ensures all internal packages can be imported without needing the pytest
namespace being set, which is critical for the initialization of xdist.
"""

import pkgutil
import subprocess
import sys
from typing import List

import _pytest
import pytest


def _modules() -> List[str]:
    pytest_pkg: str = _pytest.__path__  # type: ignore
    return sorted(
        n
        for _, n, _ in pkgutil.walk_packages(pytest_pkg, prefix=_pytest.__name__ + ".")
    )


@pytest.mark.slow
@pytest.mark.parametrize("module", _modules())
def test_no_warnings(module: str) -> None:
    # fmt: off
    subprocess.check_call((
        sys.executable,
        "-W", "error",
        "-c", f"__import__({module!r})",
    ))
    # fmt: on
