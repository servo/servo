import asyncio

import pytest


class CustomSelectorLoop(asyncio.SelectorEventLoop):
    """A subclass with no overrides, just to test for presence."""


loop = CustomSelectorLoop()


@pytest.fixture(scope="module")
def event_loop():
    """Create an instance of the default event loop for each test case."""
    yield loop
    loop.close()
