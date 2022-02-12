import math
import time

import pytest

from . import assert_console_entry


@pytest.mark.asyncio
@pytest.mark.parametrize("log_argument, expected_text", [
    ("'TEST'", "TEST"),
    ("'TWO', 'PARAMETERS'", "TWO PARAMETERS"),
    ("{}", "[object Object]"),
    ("['1', '2', '3']", "1,2,3"),
    ("null, undefined", "null undefined"),
], ids=[
    'single string',
    'two strings',
    'empty object',
    'array of strings',
    'null and undefined',
])
async def test_text_with_argument_variation(bidi_session,
                                            current_session,
                                            wait_for_event,
                                            log_argument,
                                            expected_text):
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    # TODO: To be replaced with the BiDi implementation of execute_script.
    current_session.execute_script(f"console.log({log_argument})")

    event_data = await on_entry_added

    assert_console_entry(event_data, text=expected_text)


@pytest.mark.asyncio
@pytest.mark.parametrize("log_method, expected_level", [
    ("assert", "error"),
    ("debug", "debug"),
    ("error", "error"),
    ("info", "info"),
    ("log", "info"),
    ("table", "info"),
    ("trace", "debug"),
    ("warn", "warning"),
])
async def test_level(bidi_session,
                     current_session,
                     wait_for_event,
                     log_method,
                     expected_level):
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    # TODO: To be replaced with the BiDi implementation of execute_script.
    if log_method == 'assert':
        # assert has to be called with a first falsy argument to trigger a log.
        current_session.execute_script("console.assert(false, 'foo')")
    else:
        current_session.execute_script(f"console.{log_method}('foo')")

    event_data = await on_entry_added

    assert_console_entry(event_data, text="foo", level=expected_level, method=log_method)


@pytest.mark.asyncio
async def test_timestamp(bidi_session, current_session, wait_for_event):
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    time_start = math.floor(time.time() * 1000)

    # TODO: To be replaced with the BiDi implementation of execute_async_script.
    current_session.execute_async_script("""
        const resolve = arguments[0];
        setTimeout(() => {
            console.log('foo');
            resolve();
        }, 100);
        """)

    event_data = await on_entry_added

    time_end = math.ceil(time.time() * 1000)

    assert_console_entry(event_data, text="foo", time_start=time_start, time_end=time_end)


@pytest.mark.asyncio
@pytest.mark.parametrize("new_context_method_name", ["refresh", "new_window"])
async def test_new_context(bidi_session,
                           current_session,
                           wait_for_event,
                           new_context_method_name):
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    current_session.execute_script("console.log('foo')")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="foo")

    new_context_method = getattr(current_session, new_context_method_name)
    new_context_method()

    on_entry_added = wait_for_event("log.entryAdded")
    current_session.execute_script("console.log('foo_after_refresh')")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="foo_after_refresh")
