import asyncio

import pytest

from tests.support.sync import AsyncPoll

from .. import (
    assert_before_request_sent_event,
    assert_response_event,
    PAGE_EMPTY_HTML,
    PAGE_EMPTY_TEXT,
    BEFORE_REQUEST_SENT_EVENT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)


@pytest.mark.asyncio
async def test_cors_preflight_request(bidi_session, url, fetch, setup_network_test):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_COMPLETED_EVENT,
            RESPONSE_STARTED_EVENT,
        ],
        test_url=url(PAGE_EMPTY_HTML),
    )

    # Track all received network.beforeRequestSent, responseStarted &
    # responseCompleted events in the events array.
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_before_request_sent_listener = bidi_session.add_event_listener(
        BEFORE_REQUEST_SENT_EVENT, on_event
    )
    remove_response_completed_listener = bidi_session.add_event_listener(
        RESPONSE_COMPLETED_EVENT, on_event
    )
    remove_response_started_listener = bidi_session.add_event_listener(
        RESPONSE_STARTED_EVENT, on_event
    )

    fetch_url = url(
        "/webdriver/tests/support/http_handlers/headers.py?"
        + "header=Access-Control-Allow-Origin:*&header=Access-Control-Allow-Headers:Content-Type",
        domain="alt",
    )
    asyncio.ensure_future(
        fetch(fetch_url, method="GET", headers={"Content-Type": "custom/type"})
    )

    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 6)

    # Check that all events for the CORS preflight request are received before
    # receiving events for the actual request

    # Preflight beforeRequestSent
    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "OPTIONS", "url": fetch_url},
    )
    # Preflight responseStarted
    assert_response_event(
        events[1],
        expected_request={"method": "OPTIONS", "url": fetch_url},
    )
    # Preflight responseCompleted
    assert_response_event(
        events[2],
        expected_request={"method": "OPTIONS", "url": fetch_url},
    )
    # Actual request beforeRequestSent
    assert_before_request_sent_event(
        events[3],
        expected_request={"method": "GET", "url": fetch_url},
    )
    # Actual request responseStarted
    assert_response_event(
        events[4],
        expected_request={"method": "GET", "url": fetch_url},
    )
    # Actual request responseCompleted
    assert_response_event(
        events[5],
        expected_request={"method": "GET", "url": fetch_url},
    )

    remove_before_request_sent_listener()
    remove_response_completed_listener()
    remove_response_started_listener()


@pytest.mark.asyncio
async def test_iframe_navigation_request(
    bidi_session,
    top_context,
    subscribe_events,
    setup_network_test,
    inline,
    test_page,
    test_page_cross_origin,
    test_page_same_origin_frame,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[top_context["context"]],
    )

    navigation_events = []

    async def on_event(method, data):
        navigation_events.append(data)

    remove_listener = bidi_session.add_event_listener(
        "browsingContext.navigationStarted", on_event
    )
    await subscribe_events(events=["browsingContext.navigationStarted"])

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_same_origin_frame, wait="complete"
    )

    # Get the frame_context loaded in top_context
    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    assert len(contexts[0]["children"]) == 1
    frame_context = contexts[0]["children"][0]

    assert len(navigation_events) == 2
    assert len(network_events[BEFORE_REQUEST_SENT_EVENT]) == 2
    assert len(network_events[RESPONSE_STARTED_EVENT]) == 2
    assert len(network_events[RESPONSE_COMPLETED_EVENT]) == 2

    # Check that 2 distinct navigations were captured, for the expected contexts
    assert navigation_events[0]["navigation"] == result["navigation"]
    assert navigation_events[0]["context"] == top_context["context"]
    assert navigation_events[1]["navigation"] != result["navigation"]
    assert navigation_events[1]["context"] == frame_context["context"]

    # Helper to assert the 3 main network events for this test
    def assert_events(event_index, url, context, navigation):
        expected_request = {"method": "GET", "url": url}
        expected_response = {"url": url}
        assert_before_request_sent_event(
            network_events[BEFORE_REQUEST_SENT_EVENT][event_index],
            expected_request=expected_request,
            context=context,
            navigation=navigation,
        )
        assert_response_event(
            network_events[RESPONSE_STARTED_EVENT][event_index],
            expected_response=expected_response,
            context=context,
            navigation=navigation,
        )
        assert_response_event(
            network_events[RESPONSE_COMPLETED_EVENT][event_index],
            expected_response=expected_response,
            context=context,
            navigation=navigation,
        )

    assert_events(
        0,
        url=test_page_same_origin_frame,
        context=top_context["context"],
        navigation=navigation_events[0]["navigation"],
    )
    assert_events(
        1,
        url=test_page,
        context=frame_context["context"],
        navigation=navigation_events[1]["navigation"],
    )

    # Navigate the iframe to another url
    result = await bidi_session.browsing_context.navigate(
        context=frame_context["context"], url=test_page_cross_origin, wait="complete"
    )

    assert len(navigation_events) == 3
    assert len(network_events[BEFORE_REQUEST_SENT_EVENT]) == 3
    assert len(network_events[RESPONSE_STARTED_EVENT]) == 3
    assert len(network_events[RESPONSE_COMPLETED_EVENT]) == 3
    assert_events(
        2,
        url=test_page_cross_origin,
        context=frame_context["context"],
        navigation=navigation_events[2]["navigation"],
    )


@pytest.mark.asyncio
async def test_same_navigation_id(
    bidi_session, top_context, wait_for_event, wait_for_future_safe, url, setup_network_test
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[top_context["context"]],
    )

    html_url = url(PAGE_EMPTY_HTML)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await wait_for_future_safe(on_response_completed)

    assert len(network_events[BEFORE_REQUEST_SENT_EVENT]) == 1
    assert len(network_events[RESPONSE_STARTED_EVENT]) == 1
    assert len(network_events[RESPONSE_COMPLETED_EVENT]) == 1
    expected_request = {"method": "GET", "url": html_url}
    expected_response = {"url": html_url}
    assert_before_request_sent_event(
        network_events[BEFORE_REQUEST_SENT_EVENT][0],
        expected_request=expected_request,
        context=top_context["context"],
        navigation=result["navigation"],
    )
    assert_response_event(
        network_events[RESPONSE_STARTED_EVENT][0],
        expected_response=expected_response,
        context=top_context["context"],
        navigation=result["navigation"],
    )
    assert_response_event(
        network_events[RESPONSE_COMPLETED_EVENT][0],
        expected_response=expected_response,
        context=top_context["context"],
        navigation=result["navigation"],
    )


@pytest.mark.asyncio
async def test_same_request_id(wait_for_event, wait_for_future_safe, url, setup_network_test, fetch):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    before_request_sent_events = network_events[BEFORE_REQUEST_SENT_EVENT]
    response_started_events = network_events[RESPONSE_STARTED_EVENT]
    response_completed_events = network_events[RESPONSE_COMPLETED_EVENT]

    text_url = url(PAGE_EMPTY_TEXT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(text_url)
    await wait_for_future_safe(on_response_completed)

    assert len(before_request_sent_events) == 1
    assert len(response_started_events) == 1
    assert len(response_completed_events) == 1
    expected_request = {"method": "GET", "url": text_url}
    assert_before_request_sent_event(
        before_request_sent_events[0], expected_request=expected_request
    )

    expected_response = {"url": text_url}
    assert_response_event(
        response_started_events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )
    assert_response_event(
        response_completed_events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    assert (
        before_request_sent_events[0]["request"]["request"] == response_started_events[0]["request"]["request"]
    )

    assert (
        before_request_sent_events[0]["request"]["request"] == response_completed_events[0]["request"]["request"]
    )


@pytest.mark.asyncio
async def test_subscribe_to_one_context(
    bidi_session, top_context, wait_for_event, wait_for_future_safe, url, fetch, setup_network_test
):
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=other_context["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[top_context["context"]],
    )

    # Perform a fetch request in the subscribed context and wait for the response completed event.
    text_url = url(PAGE_EMPTY_TEXT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(text_url, context=top_context)
    await wait_for_future_safe(on_response_completed)

    assert len(network_events[BEFORE_REQUEST_SENT_EVENT]) == 1
    assert len(network_events[RESPONSE_STARTED_EVENT]) == 1
    assert len(network_events[RESPONSE_COMPLETED_EVENT]) == 1

    # Check the received events have the correct context.
    expected_request = {"method": "GET", "url": text_url}
    expected_response = {"url": text_url}
    assert_before_request_sent_event(
        network_events[BEFORE_REQUEST_SENT_EVENT][0],
        expected_request=expected_request,
        context=top_context["context"],
    )
    assert_response_event(
        network_events[RESPONSE_STARTED_EVENT][0],
        expected_response=expected_response,
        context=top_context["context"],
    )
    assert_response_event(
        network_events[RESPONSE_COMPLETED_EVENT][0],
        expected_response=expected_response,
        context=top_context["context"],
    )

    # Perform another fetch request in the other context.
    await fetch(text_url, context=other_context)
    await asyncio.sleep(0.5)

    # Check that no other event was received.
    assert len(network_events[BEFORE_REQUEST_SENT_EVENT]) == 1
    assert len(network_events[RESPONSE_STARTED_EVENT]) == 1
    assert len(network_events[RESPONSE_COMPLETED_EVENT]) == 1


@pytest.mark.asyncio
async def test_event_order_with_redirect(
    bidi_session, top_context, subscribe_events, url, fetch
):
    events = [
        BEFORE_REQUEST_SENT_EVENT,
        RESPONSE_STARTED_EVENT,
        RESPONSE_COMPLETED_EVENT,
    ]
    await subscribe_events(events=events, contexts=[top_context["context"]])

    network_events = []
    listeners = []
    response_completed_events = []
    for event in events:

        async def on_event(method, data, event=event):
            network_events.append({"event": event, "url": data["request"]["url"]})

            if event == RESPONSE_COMPLETED_EVENT:
                response_completed_events.append(data)

        listeners.append(bidi_session.add_event_listener(event, on_event))

    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={text_url}"
    )

    await fetch(redirect_url, method="GET")

    # Wait until we receive two events, one for the initial request and one for
    # the redirection.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(response_completed_events) >= 2)

    events_in_expected_order = [
        {"event": "network.beforeRequestSent", "url": redirect_url},
        {"event": "network.responseStarted", "url": redirect_url},
        {"event": "network.responseCompleted", "url": redirect_url},
        {"event": "network.beforeRequestSent", "url": text_url},
        {"event": "network.responseStarted", "url": text_url},
        {"event": "network.responseCompleted", "url": text_url},
    ]

    for index in range(len(events_in_expected_order)):
        assert events_in_expected_order[index] == network_events[index]

    # cleanup
    for remove_listener in listeners:
        remove_listener()
