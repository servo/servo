"""Tests for using subprocesses in tests."""
import asyncio.subprocess
import sys

import pytest

if sys.platform == "win32":
    # The default asyncio event loop implementation on Windows does not
    # support subprocesses. Subprocesses are available for Windows if a
    # ProactorEventLoop is used.
    @pytest.yield_fixture()
    def event_loop():
        loop = asyncio.ProactorEventLoop()
        yield loop
        loop.close()


@pytest.mark.skipif(
    sys.version_info < (3, 8),
    reason="""
        When run with Python 3.7 asyncio.subprocess.create_subprocess_exec seems to be
        affected by an issue that prevents correct cleanup. Tests using pytest-trio
        will report that signal handling is already performed by another library and
        fail. [1] This is possibly a bug in CPython 3.7, so we ignore this test for
        that Python version.

        [1] https://github.com/python-trio/pytest-trio/issues/126
    """,
)
@pytest.mark.asyncio
async def test_subprocess(event_loop):
    """Starting a subprocess should be possible."""
    proc = await asyncio.subprocess.create_subprocess_exec(
        sys.executable, "--version", stdout=asyncio.subprocess.PIPE
    )
    await proc.communicate()
