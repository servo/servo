import asyncio
from typing import Any, Mapping

import pytest
import webdriver


@pytest.fixture
async def new_tab(bidi_session, current_session):
    # Open and focus a new tab to run the test in a foreground tab.
    context_id = current_session.new_window(type_hint="tab")
    initial_window = current_session.window_handle
    current_session.window_handle = context_id

    # Retrieve the browsing context info for the new tab
    contexts = await bidi_session.browsing_context.get_tree(root=context_id, max_depth=0)
    yield contexts[0]

    # Restore the focus and current window for the WebDriver session before
    # closing the tab.
    current_session.window_handle = initial_window
    await bidi_session.browsing_context.close(context=contexts[0]["context"])


@pytest.fixture
def send_blocking_command(bidi_session):
    """Send a blocking command that awaits until the BiDi response has been received."""
    async def send_blocking_command(command: str, params: Mapping[str, Any]) -> Mapping[str, Any]:
        future_response = await bidi_session.send_command(command, params)
        return await future_response
    return send_blocking_command


@pytest.fixture
def wait_for_event(bidi_session, event_loop):
    """Wait until the BiDi session emits an event and resolve  the event data."""
    def wait_for_event(event_name: str):
        future = event_loop.create_future()

        async def on_event(method, data):
            remove_listener()
            future.set_result(data)

        remove_listener = bidi_session.add_event_listener(event_name, on_event)

        return future
    return wait_for_event
