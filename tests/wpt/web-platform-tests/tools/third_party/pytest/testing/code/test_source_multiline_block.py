# flake8: noqa
import sys

import _pytest._code


def test_getstartingblock_multiline():
    """
    This test was originally found in test_source.py, but it depends on the weird
    formatting of the ``x = A`` construct seen here and our autopep8 tool can only exclude entire
    files (it does not support excluding lines/blocks using the traditional #noqa comment yet,
    see hhatto/autopep8#307). It was considered better to just move this single test to its own
    file and exclude it from autopep8 than try to complicate things.
    """

    class A(object):

        def __init__(self, *args):
            frame = sys._getframe(1)
            self.source = _pytest._code.Frame(frame).statement

    # fmt: off
    x = A('x',
          'y'
          ,
          'z')
    # fmt: on
    values = [i for i in x.source.lines if i.strip()]
    assert len(values) == 4
