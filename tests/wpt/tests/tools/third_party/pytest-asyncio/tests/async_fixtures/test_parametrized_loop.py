import asyncio

import pytest

TESTS_COUNT = 0


def teardown_module():
    # parametrized 2 * 2 times: 2 for 'event_loop' and 2 for 'fix'
    assert TESTS_COUNT == 4


@pytest.fixture(scope="module", params=[1, 2])
def event_loop(request):
    request.param
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture(params=["a", "b"])
async def fix(request):
    await asyncio.sleep(0)
    return request.param


@pytest.mark.asyncio
async def test_parametrized_loop(fix):
    await asyncio.sleep(0)
    global TESTS_COUNT
    TESTS_COUNT += 1
