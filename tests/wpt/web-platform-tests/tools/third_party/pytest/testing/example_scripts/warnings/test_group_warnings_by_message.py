# -*- coding: utf-8 -*-
import warnings

import pytest


def func():
    warnings.warn(UserWarning("foo"))


@pytest.mark.parametrize("i", range(5))
def test_foo(i):
    func()


def test_bar():
    func()
