"""Test the event loop fixture provides a separate loop for each test.

These tests need to be run together.
"""
import asyncio

import pytest

loop: asyncio.AbstractEventLoop


def test_1():
    global loop
    # The main thread should have a default event loop.
    loop = asyncio.get_event_loop_policy().get_event_loop()


@pytest.mark.asyncio
async def test_2():
    global loop
    running_loop = asyncio.get_event_loop_policy().get_event_loop()
    # Make sure this test case received a different loop
    assert running_loop is not loop
    loop = running_loop  # Store the loop reference for later


def test_3():
    global loop
    current_loop = asyncio.get_event_loop_policy().get_event_loop()
    # Now the event loop from test_2 should have been cleaned up
    assert loop is not current_loop


def test_4(event_loop):
    # If a test sets the loop to None -- pytest_fixture_post_finalizer()
    # still should work
    asyncio.get_event_loop_policy().set_event_loop(None)
