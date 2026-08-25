"""Unit tests for overriding the event loop."""
import asyncio

import pytest


@pytest.mark.asyncio
async def test_for_custom_loop():
    """This test should be executed using the custom loop."""
    await asyncio.sleep(0.01)
    assert type(asyncio.get_event_loop()).__name__ == "CustomSelectorLoop"


@pytest.mark.asyncio
async def test_dependent_fixture(dependent_fixture):
    await asyncio.sleep(0.1)
