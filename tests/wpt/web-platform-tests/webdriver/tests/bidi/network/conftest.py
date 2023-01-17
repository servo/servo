import json

import pytest

from webdriver.bidi.modules.script import ContextTarget

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"


@pytest.fixture
def fetch(bidi_session, top_context):
    """Perform a fetch from the page of the top level context."""
    async def fetch(url, method="GET", headers=None):
        method_arg = f"method: '{method}',"

        headers_arg = ""
        if headers != None:
            headers_arg = f"headers: {json.dumps(headers)},"

        # Wait for fetch() to resolve a response and for response.text() to
        # resolve as well to make sure the request/response is completed when
        # the helper returns.
        await bidi_session.script.evaluate(
            expression=f"""
                 fetch("{url}", {{
                   {method_arg}
                   {headers_arg}
                 }}).then(response => response.text());""",
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        )

    return fetch


@pytest.fixture
async def setup_network_test(bidi_session, subscribe_events, top_context, url):
    """Navigate the current top level context to the provided url and subscribe
    to network.beforeRequestSent.

    Returns an `events` dictionary in which the captured network events will be added.
    The keys of the dictionary are network event names (eg. "network.beforeRequestSent"),
    and the value is an array of collected events.
    """
    listeners = []

    async def _setup_network_test(events, test_url=url(PAGE_EMPTY_HTML)):
        nonlocal listeners

        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=test_url,
            wait="complete",
        )
        await subscribe_events(events)

        network_events = {}
        for event in events:
            network_events[event] = []
            async def on_event(method, data, event=event):
                network_events[event].append(data)
            listeners.append(bidi_session.add_event_listener(event, on_event))

        return network_events

    yield _setup_network_test

    # cleanup
    for remove_listener in listeners:
        remove_listener()
