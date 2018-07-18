from __future__ import absolute_import, division, print_function
import sys

import pytest
from _pytest.compat import is_generator, get_real_func, safe_getattr
from _pytest.outcomes import OutcomeException


def test_is_generator():

    def zap():
        yield

    def foo():
        pass

    assert is_generator(zap)
    assert not is_generator(foo)


def test_real_func_loop_limit():

    class Evil(object):

        def __init__(self):
            self.left = 1000

        def __repr__(self):
            return "<Evil left={left}>".format(left=self.left)

        def __getattr__(self, attr):
            if not self.left:
                raise RuntimeError("its over")
            self.left -= 1
            return self

    evil = Evil()

    with pytest.raises(ValueError):
        res = get_real_func(evil)
        print(res)


@pytest.mark.skipif(
    sys.version_info < (3, 4), reason="asyncio available in Python 3.4+"
)
def test_is_generator_asyncio(testdir):
    testdir.makepyfile(
        """
        from _pytest.compat import is_generator
        import asyncio
        @asyncio.coroutine
        def baz():
            yield from [1,2,3]

        def test_is_generator_asyncio():
            assert not is_generator(baz)
    """
    )
    # avoid importing asyncio into pytest's own process,
    # which in turn imports logging (#8)
    result = testdir.runpytest_subprocess()
    result.stdout.fnmatch_lines(["*1 passed*"])


@pytest.mark.skipif(
    sys.version_info < (3, 5), reason="async syntax available in Python 3.5+"
)
def test_is_generator_async_syntax(testdir):
    testdir.makepyfile(
        """
        from _pytest.compat import is_generator
        def test_is_generator_py35():
            async def foo():
                await foo()

            async def bar():
                pass

            assert not is_generator(foo)
            assert not is_generator(bar)
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


class ErrorsHelper(object):

    @property
    def raise_exception(self):
        raise Exception("exception should be catched")

    @property
    def raise_fail(self):
        pytest.fail("fail should be catched")


def test_helper_failures():
    helper = ErrorsHelper()
    with pytest.raises(Exception):
        helper.raise_exception
    with pytest.raises(OutcomeException):
        helper.raise_fail


def test_safe_getattr():
    helper = ErrorsHelper()
    assert safe_getattr(helper, "raise_exception", "default") == "default"
    assert safe_getattr(helper, "raise_fail", "default") == "default"
