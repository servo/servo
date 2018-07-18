import pytest

from _pytest.python import PyCollector


class PyCollectorMock(PyCollector):
    """evil hack"""

    def __init__(self):
        self.called = False

    def _makeitem(self, *k):
        """hack to disable the actual behaviour"""
        self.called = True


def test_pycollector_makeitem_is_deprecated():

    collector = PyCollectorMock()
    with pytest.deprecated_call():
        collector.makeitem("foo", "bar")
    assert collector.called
