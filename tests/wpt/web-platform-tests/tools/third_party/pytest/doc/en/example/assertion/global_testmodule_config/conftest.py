import os.path

import pytest

mydir = os.path.dirname(__file__)


def pytest_runtest_setup(item):
    if isinstance(item, pytest.Function):
        if not item.fspath.relto(mydir):
            return
        mod = item.getparent(pytest.Module).obj
        if hasattr(mod, "hello"):
            print(f"mod.hello {mod.hello!r}")
