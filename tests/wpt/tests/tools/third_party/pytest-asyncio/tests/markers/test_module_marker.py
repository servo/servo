"""Test if pytestmark works when defined in a module."""
import asyncio

import pytest

pytestmark = pytest.mark.asyncio


class TestPyTestMark:
    async def test_is_asyncio(self, event_loop, sample_fixture):
        assert asyncio.get_event_loop()

        counter = 1

        async def inc():
            nonlocal counter
            counter += 1
            await asyncio.sleep(0)

        await asyncio.ensure_future(inc())
        assert counter == 2


async def test_is_asyncio(event_loop, sample_fixture):
    assert asyncio.get_event_loop()
    counter = 1

    async def inc():
        nonlocal counter
        counter += 1
        await asyncio.sleep(0)

    await asyncio.ensure_future(inc())
    assert counter == 2


@pytest.fixture
def sample_fixture():
    return None
