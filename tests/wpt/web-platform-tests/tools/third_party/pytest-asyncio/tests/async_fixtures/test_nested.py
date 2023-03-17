import asyncio

import pytest


@pytest.fixture()
async def async_inner_fixture():
    await asyncio.sleep(0.01)
    print("inner start")
    yield True
    print("inner stop")


@pytest.fixture()
async def async_fixture_outer(async_inner_fixture, event_loop):
    await asyncio.sleep(0.01)
    print("outer start")
    assert async_inner_fixture is True
    yield True
    print("outer stop")


@pytest.mark.asyncio
async def test_async_fixture(async_fixture_outer):
    assert async_fixture_outer is True
    print("test_async_fixture")
