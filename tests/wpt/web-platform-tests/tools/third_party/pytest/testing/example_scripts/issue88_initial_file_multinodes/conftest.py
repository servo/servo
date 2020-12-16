# -*- coding: utf-8 -*-
import pytest


class MyFile(pytest.File):
    def collect(self):
        return [MyItem("hello", parent=self)]


def pytest_collect_file(path, parent):
    return MyFile(path, parent)


class MyItem(pytest.Item):
    pass
