# mypy: allow-untyped-defs
import pytest


class MyFile(pytest.File):
    def collect(self):
        return [MyItem.from_parent(name="hello", parent=self)]


def pytest_collect_file(file_path, parent):
    return MyFile.from_parent(path=file_path, parent=parent)


class MyItem(pytest.Item):
    def runtest(self):
        raise NotImplementedError()
