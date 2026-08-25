from textwrap import dedent


def test_strict_mode_ignores_trio_fixtures(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import pytest
        import pytest_asyncio
        import pytest_trio

        pytest_plugins = ["pytest_asyncio", "pytest_trio"]

        @pytest_trio.trio_fixture
        async def any_fixture():
            return True

        @pytest.mark.trio
        async def test_anything(any_fixture):
            pass
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=strict")
    result.assert_outcomes(passed=1)
