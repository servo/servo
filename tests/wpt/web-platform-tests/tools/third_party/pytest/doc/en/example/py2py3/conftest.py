import sys
import pytest

py3 = sys.version_info[0] >= 3

class DummyCollector(pytest.collect.File):
    def collect(self):
        return []

def pytest_pycollect_makemodule(path, parent):
    bn = path.basename
    if "py3" in bn and not py3 or ("py2" in bn and py3):
        return DummyCollector(path, parent=parent)



