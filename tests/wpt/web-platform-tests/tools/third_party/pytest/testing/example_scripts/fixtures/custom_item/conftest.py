# -*- coding: utf-8 -*-
import pytest


class CustomItem(pytest.Item, pytest.File):
    def runtest(self):
        pass


def pytest_collect_file(path, parent):
    return CustomItem(path, parent)
