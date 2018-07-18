from __future__ import absolute_import, division, print_function
import pkg_resources

import pytest


@pytest.mark.parametrize("entrypoint", ["py.test", "pytest"])
def test_entry_point_exist(entrypoint):
    assert entrypoint in pkg_resources.get_entry_map("pytest")["console_scripts"]


def test_pytest_entry_points_are_identical():
    entryMap = pkg_resources.get_entry_map("pytest")["console_scripts"]
    assert entryMap["pytest"].module_name == entryMap["py.test"].module_name
