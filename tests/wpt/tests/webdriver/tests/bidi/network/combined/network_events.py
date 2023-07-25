import asyncio

import pytest

from .. import (
    assert_before_request_sent_event,
    assert_response_event,
)

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"


@pytest.mark.asyncio
async def test_same_navigation_id(
    bidi_session, top_context, wait_for_event, url, setup_network_test
):
    network_events = await setup_network_test(
        events=[
            "network.beforeRequestSent",
            "network.responseStarted",
            "network.responseCompleted",
        ],
        contexts=[top_context["context"]],
    )

    html_url = url(PAGE_EMPTY_HTML)
    on_response_completed = wait_for_event("network.responseCompleted")
    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await on_response_completed

    assert len(network_events["network.beforeRequestSent"]) == 1
    assert len(network_events["network.responseStarted"]) == 1
    assert len(network_events["network.responseCompleted"]) == 1
    expected_request = {"method": "GET", "url": html_url}
    expected_response = {"url": html_url}
    assert_before_request_sent_event(
        network_events["network.beforeRequestSent"][0],
        expected_request=expected_request,
        context=top_context["context"],
        navigation=result["navigation"],
    )
    assert_response_event(
        network_events["network.responseStarted"][0],
        expected_response=expected_response,
        context=top_context["context"],
        navigation=result["navigation"],
    )
    assert_response_event(
        network_events["network.responseCompleted"][0],
        expected_response=expected_response,
        context=top_context["context"],
        navigation=result["navigation"],
    )


@pytest.mark.asyncio
async def test_same_request_id(wait_for_event, url, setup_network_test, fetch):
    network_events = await setup_network_test(
        events=[
            "network.beforeRequestSent",
            "network.responseStarted",
            "network.responseCompleted",
        ]
    )
    before_request_sent_events = network_events["network.beforeRequestSent"]
    response_started_events = network_events["network.responseStarted"]
    response_completed_events = network_events["network.responseCompleted"]

    text_url = url(PAGE_EMPTY_TEXT)
    on_response_completed = wait_for_event("network.responseCompleted")
    await fetch(text_url)
    await on_response_completed

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
        before_request_sent_events[0]["request"]["request"]
        == response_started_events[0]["request"]["request"]
    )

    assert (
        before_request_sent_events[0]["request"]["request"]
        == response_completed_events[0]["request"]["request"]
    )


@pytest.mark.asyncio
async def test_subscribe_to_one_context(
    bidi_session, top_context, wait_for_event, url, fetch, setup_network_test
):
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=other_context["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    network_events = await setup_network_test(
        events=[
            "network.beforeRequestSent",
            "network.responseStarted",
            "network.responseCompleted",
        ],
        contexts=[top_context["context"]],
    )

    # Perform a fetch request in the subscribed context and wait for the response completed event.
    text_url = url(PAGE_EMPTY_TEXT)
    on_response_completed = wait_for_event("network.responseCompleted")
    await fetch(text_url, context=top_context)
    await on_response_completed

    assert len(network_events["network.beforeRequestSent"]) == 1
    assert len(network_events["network.responseStarted"]) == 1
    assert len(network_events["network.responseCompleted"]) == 1

    # Check the received events have the correct context.
    expected_request = {"method": "GET", "url": text_url}
    expected_response = {"url": text_url}
    assert_before_request_sent_event(
        network_events["network.beforeRequestSent"][0],
        expected_request=expected_request,
        context=top_context["context"],
    )
    assert_response_event(
        network_events["network.responseStarted"][0],
        expected_response=expected_response,
        context=top_context["context"],
    )
    assert_response_event(
        network_events["network.responseCompleted"][0],
        expected_response=expected_response,
        context=top_context["context"],
    )

    # Perform another fetch request in the other context.
    await fetch(text_url, context=other_context)
    await asyncio.sleep(0.5)

    # Check that no other event was received.
    assert len(network_events["network.beforeRequestSent"]) == 1
    assert len(network_events["network.responseStarted"]) == 1
    assert len(network_events["network.responseCompleted"]) == 1
