"""Tests for the Hypothesis integration, which wraps async functions in a
sync shim for Hypothesis.
"""
import asyncio
from textwrap import dedent

import pytest
from hypothesis import given, strategies as st


@pytest.fixture(scope="module")
def event_loop():
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@given(st.integers())
@pytest.mark.asyncio
async def test_mark_inner(n):
    assert isinstance(n, int)


@pytest.mark.asyncio
@given(st.integers())
async def test_mark_outer(n):
    assert isinstance(n, int)


@pytest.mark.parametrize("y", [1, 2])
@given(x=st.none())
@pytest.mark.asyncio
async def test_mark_and_parametrize(x, y):
    assert x is None
    assert y in (1, 2)


@given(st.integers())
@pytest.mark.asyncio
async def test_can_use_fixture_provided_event_loop(event_loop, n):
    semaphore = asyncio.Semaphore(value=0)
    event_loop.call_soon(semaphore.release)
    await semaphore.acquire()


def test_async_auto_marked(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest
        from hypothesis import given
        import hypothesis.strategies as st

        pytest_plugins = 'pytest_asyncio'

        @given(n=st.integers())
        async def test_hypothesis(n: int):
            assert isinstance(n, int)
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)


def test_sync_not_auto_marked(testdir):
    """Assert that synchronous Hypothesis functions are not marked with asyncio"""
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest
        from hypothesis import given
        import hypothesis.strategies as st

        pytest_plugins = 'pytest_asyncio'

        @given(n=st.integers())
        def test_hypothesis(request, n: int):
            markers = [marker.name for marker in request.node.own_markers]
            assert "asyncio" not in markers
            assert isinstance(n, int)
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)
