from textwrap import dedent


def test_auto_mode_cmdline(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest

        pytest_plugins = 'pytest_asyncio'

        async def test_a():
            await asyncio.sleep(0)
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)


def test_auto_mode_cfg(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest

        pytest_plugins = 'pytest_asyncio'

        async def test_a():
            await asyncio.sleep(0)
        """
        )
    )
    testdir.makefile(".ini", pytest="[pytest]\nasyncio_mode = auto\n")
    result = testdir.runpytest()
    result.assert_outcomes(passed=1)


def test_auto_mode_async_fixture(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest

        pytest_plugins = 'pytest_asyncio'

        @pytest.fixture
        async def fixture_a():
            await asyncio.sleep(0)
            return 1

        async def test_a(fixture_a):
            await asyncio.sleep(0)
            assert fixture_a == 1
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)


def test_auto_mode_method_fixture(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest

        pytest_plugins = 'pytest_asyncio'


        class TestA:

            @pytest.fixture
            async def fixture_a(self):
                await asyncio.sleep(0)
                return 1

            async def test_a(self, fixture_a):
                await asyncio.sleep(0)
                assert fixture_a == 1
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)


def test_auto_mode_static_method(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio

        pytest_plugins = 'pytest_asyncio'


        class TestA:

            @staticmethod
            async def test_a():
                await asyncio.sleep(0)
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)


def test_auto_mode_static_method_fixture(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import asyncio
        import pytest

        pytest_plugins = 'pytest_asyncio'


        class TestA:

            @staticmethod
            @pytest.fixture
            async def fixture_a():
                await asyncio.sleep(0)
                return 1

            @staticmethod
            async def test_a(fixture_a):
                await asyncio.sleep(0)
                assert fixture_a == 1
        """
        )
    )
    result = testdir.runpytest("--asyncio-mode=auto")
    result.assert_outcomes(passed=1)
