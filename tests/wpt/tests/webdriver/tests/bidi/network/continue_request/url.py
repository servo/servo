import pytest

from webdriver.bidi.modules.script import ContextTarget

from .. import (
    assert_response_event,
    BEFORE_REQUEST_SENT_EVENT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)

pytestmark = pytest.mark.asyncio

CONTEXT_LOAD_EVENT = "browsingContext.load"


async def test_navigation(
    bidi_session,
    subscribe_events,
    setup_blocked_request,
    top_context,
    url,
    inline,
    wait_for_event,
    wait_for_future_safe,
):
    initial_url = inline("<div id='from-initial'>initial</div>")
    redirect_url = inline("<div id='from-redirect'>redirect</div>")

    request = await setup_blocked_request(
        phase="beforeRequestSent",
        navigate=True,
        blocked_url=initial_url,
    )

    # Collect all beforeRequestSent, responseStarted and responseCompleted
    # events for the rest of the test.
    events = []
    async def on_event(method, data):
        events.append([method, data])

    remove_before_request_sent_listener = bidi_session.add_event_listener(
        BEFORE_REQUEST_SENT_EVENT, on_event
    )
    remove_response_completed_listener = bidi_session.add_event_listener(
        RESPONSE_COMPLETED_EVENT, on_event
    )
    remove_response_started_listener = bidi_session.add_event_listener(
        RESPONSE_STARTED_EVENT, on_event
    )

    # Note: only subscribe to network events after setup_blocked_request is done
    # otherwise the event subscription required to setup the blocked request
    # will collide with the global subscription here.
    await subscribe_events(
        events=[
            CONTEXT_LOAD_EVENT,
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_COMPLETED_EVENT,
            RESPONSE_STARTED_EVENT,
        ]
    )

    on_load = wait_for_event(CONTEXT_LOAD_EVENT)
    await bidi_session.network.continue_request(request=request, url=redirect_url)
    event = await wait_for_future_safe(on_load)

    # Check the node from the initial url is not available in the page.
    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": "css", "value": "#from-initial"},
    )
    assert len(result["nodes"]) == 0

    # Check the node from the redirected url is available in the page.
    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": "css", "value": "#from-redirect"},
    )
    assert len(result["nodes"]) == 1

    # Check that the window.location remains on initial_url
    result = await bidi_session.script.evaluate(
        expression="window.location.href",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result["value"] == initial_url

    # Check that only the expected number of events were received.
    assert len(events) == 2

    expected_request = {"method": "GET", "url": redirect_url}
    expected_response = {"url": redirect_url}

    assert events[0][0] == "network.responseStarted"
    assert_response_event(
        events[0][1],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )

    assert events[1][0] == "network.responseCompleted"
    assert_response_event(
        events[1][1],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )

    remove_before_request_sent_listener()
    remove_response_completed_listener()
    remove_response_started_listener()
