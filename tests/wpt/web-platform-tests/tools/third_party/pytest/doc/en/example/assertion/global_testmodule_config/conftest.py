# -*- coding: utf-8 -*-
import py

import pytest

mydir = py.path.local(__file__).dirpath()


def pytest_runtest_setup(item):
    if isinstance(item, pytest.Function):
        if not item.fspath.relto(mydir):
            return
        mod = item.getparent(pytest.Module).obj
        if hasattr(mod, "hello"):
            print("mod.hello {!r}".format(mod.hello))
