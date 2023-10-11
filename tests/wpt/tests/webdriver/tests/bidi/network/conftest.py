import json

import pytest
import pytest_asyncio

from webdriver.bidi.error import NoSuchInterceptException
from webdriver.bidi.modules.script import ContextTarget

RESPONSE_COMPLETED_EVENT = "network.responseCompleted"

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"


@pytest_asyncio.fixture
async def add_intercept(bidi_session):
    """Add a network intercept for the provided phases and url patterns, and
    ensure the intercept is removed at the end of the test."""

    intercepts = []
    async def add_intercept(phases, url_patterns):
        nonlocal intercepts
        intercept = await bidi_session.network.add_intercept(
            phases=phases,
            url_patterns=url_patterns,
        )
        intercepts.append(intercept)

        return intercept

    yield add_intercept

    # Remove all added intercepts at the end of the test
    for intercept in intercepts:
        try:
            await bidi_session.network.remove_intercept(intercept=intercept)
        except (NoSuchInterceptException):
            # Ignore exceptions in case a specific intercept was already removed
            # during the test.
            pass


@pytest.fixture
def fetch(bidi_session, top_context, configuration):
    """Perform a fetch from the page of the provided context, default to the
    top context.
    """
    async def fetch(url, method="GET", headers=None, context=top_context, timeout_in_seconds=3):
        method_arg = f"method: '{method}',"

        headers_arg = ""
        if headers is not None:
            headers_arg = f"headers: {json.dumps(headers)},"

        timeout_in_seconds = timeout_in_seconds * configuration["timeout_multiplier"]

        # Wait for fetch() to resolve a response and for response.text() to
        # resolve as well to make sure the request/response is completed when
        # the helper returns.
        await bidi_session.script.evaluate(
            expression=f"""
                 {{
                   const controller = new AbortController();
                   setTimeout(() => controller.abort(), {timeout_in_seconds * 1000});
                   fetch("{url}", {{
                     {method_arg}
                     {headers_arg}
                     signal: controller.signal
                   }}).then(response => response.text());
                 }}""",
            target=ContextTarget(context["context"]),
            await_promise=True,
        )

    return fetch


@pytest_asyncio.fixture
async def setup_network_test(
    bidi_session, subscribe_events, wait_for_event, top_context, url
):
    """Navigate the current top level context to the provided url and subscribe
    to network.beforeRequestSent.

    Returns an `events` dictionary in which the captured network events will be added.
    The keys of the dictionary are network event names (eg. "network.beforeRequestSent"),
    and the value is an array of collected events.
    """
    listeners = []

    async def _setup_network_test(events, test_url=url(PAGE_EMPTY_HTML), contexts=None):
        nonlocal listeners

        # Listen for network.responseCompleted for the initial navigation to
        # make sure this event will not be captured unexpectedly by the tests.
        await bidi_session.session.subscribe(
            events=[RESPONSE_COMPLETED_EVENT], contexts=[top_context["context"]]
        )
        on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=test_url,
            wait="complete",
        )
        await on_response_completed
        await bidi_session.session.unsubscribe(
            events=[RESPONSE_COMPLETED_EVENT], contexts=[top_context["context"]]
        )

        await subscribe_events(events, contexts)

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
