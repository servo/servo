import pytest


class CustomItem(pytest.Item):
    def runtest(self):
        pass


class CustomFile(pytest.File):
    def collect(self):
        yield CustomItem.from_parent(name="foo", parent=self)


def pytest_collect_file(file_path, parent):
    return CustomFile.from_parent(path=file_path, parent=parent)
