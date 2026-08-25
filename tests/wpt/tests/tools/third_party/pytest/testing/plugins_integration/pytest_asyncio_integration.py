# mypy: allow-untyped-defs
import asyncio

import pytest


@pytest.mark.asyncio
async def test_sleep():
    await asyncio.sleep(0)
