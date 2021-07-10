import trio

import pytest


@pytest.mark.trio
async def test_sleep():
    await trio.sleep(0)
