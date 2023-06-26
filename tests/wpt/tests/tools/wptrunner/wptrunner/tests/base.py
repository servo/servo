# mypy: allow-untyped-defs

import os
import sys

from os.path import dirname, join

import pytest

sys.path.insert(0, join(dirname(__file__), "..", ".."))

from .. import browsers


_products = browsers.product_list
_active_products = set()

if "CURRENT_TOX_ENV" in os.environ:
    current_tox_env_split = os.environ["CURRENT_TOX_ENV"].split("-")

    tox_env_extra_browsers = {
        "chrome": {"chrome_android"},
        "edge": {"edge_webdriver"},
        "servo": {"servodriver"},
    }

    _active_products = set(_products) & set(current_tox_env_split)
    for product in frozenset(_active_products):
        _active_products |= tox_env_extra_browsers.get(product, set())
else:
    _active_products = set(_products)


class all_products:
    def __init__(self, arg, marks={}):
        self.arg = arg
        self.marks = marks

    def __call__(self, f):
        params = []
        for product in _products:
            if product in self.marks:
                params.append(pytest.param(product, marks=self.marks[product]))
            else:
                params.append(product)
        return pytest.mark.parametrize(self.arg, params)(f)


class active_products:
    def __init__(self, arg, marks={}):
        self.arg = arg
        self.marks = marks

    def __call__(self, f):
        params = []
        for product in _products:
            if product not in _active_products:
                params.append(pytest.param(product, marks=pytest.mark.skip(reason="wrong toxenv")))
            elif product in self.marks:
                params.append(pytest.param(product, marks=self.marks[product]))
            else:
                params.append(product)
        return pytest.mark.parametrize(self.arg, params)(f)
