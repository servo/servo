"""Quick'n'dirty unit tests for provided fixtures and markers."""
import asyncio
from textwrap import dedent

import pytest

import pytest_asyncio.plugin


async def async_coro():
    await asyncio.sleep(0)
    return "ok"


def test_event_loop_fixture(event_loop):
    """Test the injection of the event_loop fixture."""
    assert event_loop
    ret = event_loop.run_until_complete(async_coro())
    assert ret == "ok"


@pytest.mark.asyncio
async def test_asyncio_marker():
    """Test the asyncio pytest marker."""
    await asyncio.sleep(0)


@pytest.mark.xfail(reason="need a failure", strict=True)
@pytest.mark.asyncio
async def test_asyncio_marker_fail():
    raise AssertionError


@pytest.mark.asyncio
async def test_asyncio_marker_with_default_param(a_param=None):
    """Test the asyncio pytest marker."""
    await asyncio.sleep(0)


@pytest.mark.asyncio
async def test_unused_port_fixture(unused_tcp_port, event_loop):
    """Test the unused TCP port fixture."""

    async def closer(_, writer):
        writer.close()

    server1 = await asyncio.start_server(closer, host="localhost", port=unused_tcp_port)

    with pytest.raises(IOError):
        await asyncio.start_server(closer, host="localhost", port=unused_tcp_port)

    server1.close()
    await server1.wait_closed()


@pytest.mark.asyncio
async def test_unused_udp_port_fixture(unused_udp_port, event_loop):
    """Test the unused TCP port fixture."""

    class Closer:
        def connection_made(self, transport):
            pass

        def connection_lost(self, *arg, **kwd):
            pass

    transport1, _ = await event_loop.create_datagram_endpoint(
        Closer,
        local_addr=("127.0.0.1", unused_udp_port),
        reuse_port=False,
    )

    with pytest.raises(IOError):
        await event_loop.create_datagram_endpoint(
            Closer,
            local_addr=("127.0.0.1", unused_udp_port),
            reuse_port=False,
        )

    transport1.abort()


@pytest.mark.asyncio
async def test_unused_port_factory_fixture(unused_tcp_port_factory, event_loop):
    """Test the unused TCP port factory fixture."""

    async def closer(_, writer):
        writer.close()

    port1, port2, port3 = (
        unused_tcp_port_factory(),
        unused_tcp_port_factory(),
        unused_tcp_port_factory(),
    )

    server1 = await asyncio.start_server(closer, host="localhost", port=port1)
    server2 = await asyncio.start_server(closer, host="localhost", port=port2)
    server3 = await asyncio.start_server(closer, host="localhost", port=port3)

    for port in port1, port2, port3:
        with pytest.raises(IOError):
            await asyncio.start_server(closer, host="localhost", port=port)

    server1.close()
    await server1.wait_closed()
    server2.close()
    await server2.wait_closed()
    server3.close()
    await server3.wait_closed()


@pytest.mark.asyncio
async def test_unused_udp_port_factory_fixture(unused_udp_port_factory, event_loop):
    """Test the unused UDP port factory fixture."""

    class Closer:
        def connection_made(self, transport):
            pass

        def connection_lost(self, *arg, **kwd):
            pass

    port1, port2, port3 = (
        unused_udp_port_factory(),
        unused_udp_port_factory(),
        unused_udp_port_factory(),
    )

    transport1, _ = await event_loop.create_datagram_endpoint(
        Closer,
        local_addr=("127.0.0.1", port1),
        reuse_port=False,
    )
    transport2, _ = await event_loop.create_datagram_endpoint(
        Closer,
        local_addr=("127.0.0.1", port2),
        reuse_port=False,
    )
    transport3, _ = await event_loop.create_datagram_endpoint(
        Closer,
        local_addr=("127.0.0.1", port3),
        reuse_port=False,
    )

    for port in port1, port2, port3:
        with pytest.raises(IOError):
            await event_loop.create_datagram_endpoint(
                Closer,
                local_addr=("127.0.0.1", port),
                reuse_port=False,
            )

    transport1.abort()
    transport2.abort()
    transport3.abort()


def test_unused_port_factory_duplicate(unused_tcp_port_factory, monkeypatch):
    """Test correct avoidance of duplicate ports."""
    counter = 0

    def mock_unused_tcp_port(_ignored):
        """Force some duplicate ports."""
        nonlocal counter
        counter += 1
        if counter < 5:
            return 10000
        else:
            return 10000 + counter

    monkeypatch.setattr(pytest_asyncio.plugin, "_unused_port", mock_unused_tcp_port)

    assert unused_tcp_port_factory() == 10000
    assert unused_tcp_port_factory() > 10000


def test_unused_udp_port_factory_duplicate(unused_udp_port_factory, monkeypatch):
    """Test correct avoidance of duplicate UDP ports."""
    counter = 0

    def mock_unused_udp_port(_ignored):
        """Force some duplicate ports."""
        nonlocal counter
        counter += 1
        if counter < 5:
            return 10000
        else:
            return 10000 + counter

    monkeypatch.setattr(pytest_asyncio.plugin, "_unused_port", mock_unused_udp_port)

    assert unused_udp_port_factory() == 10000
    assert unused_udp_port_factory() > 10000


class TestMarkerInClassBasedTests:
    """Test that asyncio marked functions work for methods of test classes."""

    @pytest.mark.asyncio
    async def test_asyncio_marker_with_explicit_loop_fixture(self, event_loop):
        """Test the "asyncio" marker works on a method in
        a class-based test with explicit loop fixture."""
        ret = await async_coro()
        assert ret == "ok"

    @pytest.mark.asyncio
    async def test_asyncio_marker_with_implicit_loop_fixture(self):
        """Test the "asyncio" marker works on a method in
        a class-based test with implicit loop fixture."""
        ret = await async_coro()
        assert ret == "ok"


class TestEventLoopStartedBeforeFixtures:
    @pytest.fixture
    async def loop(self):
        return asyncio.get_event_loop()

    @staticmethod
    def foo():
        return 1

    @pytest.mark.asyncio
    async def test_no_event_loop(self, loop):
        assert await loop.run_in_executor(None, self.foo) == 1

    @pytest.mark.asyncio
    async def test_event_loop_after_fixture(self, loop, event_loop):
        assert await loop.run_in_executor(None, self.foo) == 1

    @pytest.mark.asyncio
    async def test_event_loop_before_fixture(self, event_loop, loop):
        assert await loop.run_in_executor(None, self.foo) == 1


@pytest.mark.asyncio
async def test_no_warning_on_skip():
    pytest.skip("Test a skip error inside asyncio")


def test_async_close_loop(event_loop):
    event_loop.close()
    return "ok"


def test_warn_asyncio_marker_for_regular_func(testdir):
    testdir.makepyfile(
        dedent(
            """\
        import pytest

        pytest_plugins = 'pytest_asyncio'

        @pytest.mark.asyncio
        def test_a():
            pass
        """
        )
    )
    testdir.makefile(
        ".ini",
        pytest=dedent(
            """\
        [pytest]
        asyncio_mode = strict
        filterwarnings =
            default
    """
        ),
    )
    result = testdir.runpytest()
    result.assert_outcomes(passed=1)
    result.stdout.fnmatch_lines(
        ["*is marked with '@pytest.mark.asyncio' but it is not an async function.*"]
    )
