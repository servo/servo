"""Tests for the Flaky integration, which retries failed tests.
"""
from textwrap import dedent


def test_auto_mode_cmdline(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import flaky
        import pytest

        _threshold = -1

        @flaky.flaky(3, 2)
        @pytest.mark.asyncio
        async def test_asyncio_flaky_thing_that_fails_then_succeeds():
            global _threshold
            await asyncio.sleep(0.1)
            _threshold += 1
            assert _threshold != 1
        """
        )
    )
    # runpytest_subprocess() is required to don't pollute the output
    # with flaky restart information
    result = testdir.runpytest_subprocess("--asyncio-mode=strict")
    result.assert_outcomes(passed=1)
    result.stdout.fnmatch_lines(
        [
            "===Flaky Test Report===",
            "test_asyncio_flaky_thing_that_fails_then_succeeds passed 1 "
            "out of the required 2 times. Running test again until it passes 2 times.",
            "test_asyncio_flaky_thing_that_fails_then_succeeds failed "
            "(1 runs remaining out of 3).",
            "	<class 'AssertionError'>",
            "	assert 1 != 1",
            "test_asyncio_flaky_thing_that_fails_then_succeeds passed 2 "
            "out of the required 2 times. Success!",
            "===End Flaky Test Report===",
        ]
    )
