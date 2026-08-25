# mypy: allow-untyped-defs
import anyio

import pytest


@pytest.mark.anyio
async def test_sleep():
    await anyio.sleep(0)
