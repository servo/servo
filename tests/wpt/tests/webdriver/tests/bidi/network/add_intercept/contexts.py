import asyncio

import pytest
from webdriver.bidi.modules.script import ScriptEvaluateResultException

from .. import (
    assert_before_request_sent_event,
    PAGE_EMPTY_HTML,
    PAGE_EMPTY_TEXT,
    BEFORE_REQUEST_SENT_EVENT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)


@pytest.mark.asyncio
@pytest.mark.parametrize("phase", ["beforeRequestSent", "responseStarted"])
async def test_other_context(
    bidi_session,
    url,
    top_context,
    add_intercept,
    fetch,
    setup_network_test,
    phase,
):
    # Subscribe to network events only in top_context
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[top_context["context"]],
    )

    # Create another tab, where network events are not monitored.
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=other_context["context"], url=url(PAGE_EMPTY_HTML), wait="complete"
    )

    # Add an intercept.
    text_url = url(PAGE_EMPTY_TEXT)
    await add_intercept(
        phases=[phase],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    # Request to top_context should be blocked and throw a ScriptEvaluateResultException
    # from the AbortController.
    with pytest.raises(ScriptEvaluateResultException):
        await fetch(text_url, context=top_context)

    # Request to other_context should not be blocked.
    await fetch(text_url, context=other_context)


@pytest.mark.asyncio
async def test_other_context_with_event_subscription(
    bidi_session,
    url,
    top_context,
    add_intercept,
    fetch,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe
):
    # Create another tab that will listen to network events without interception.
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=other_context["context"], url=url(PAGE_EMPTY_HTML), wait="complete"
    )

    # Subscribe to network events in both contexts.
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[top_context["context"], other_context["context"]],
    )

    # Add an intercept to top_context only.
    text_url = url(PAGE_EMPTY_TEXT)
    await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[top_context["context"]]
    )

    # Request to the top_context should be blocked.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=top_context))
    event = await wait_for_future_safe(on_network_event)
    assert_before_request_sent_event(
        event, is_blocked=True
    )

    # Request to other_context should not be blocked.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=other_context))
    event = await wait_for_future_safe(on_network_event)
    assert_before_request_sent_event(
        event, is_blocked=False
    )


@pytest.mark.asyncio
async def test_two_contexts_same_intercept(
    bidi_session,
    url,
    top_context,
    add_intercept,
    fetch,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe
):
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=other_context["context"], url=url(PAGE_EMPTY_HTML), wait="complete"
    )

    # Subscribe to network events in both contexts.
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ],
        contexts=[top_context["context"], other_context["context"]],
    )

    # Add an intercept to both contexts
    text_url = url(PAGE_EMPTY_TEXT)
    intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[top_context["context"], other_context["context"]],
    )

    # Request on the top_context should be blocked.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=top_context))
    event = await wait_for_future_safe(on_network_event)
    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[intercept]
    )

    # Request on the other_context should be blocked.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=other_context))
    event = await wait_for_future_safe(on_network_event)
    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[intercept]
    )


@pytest.mark.asyncio
async def test_two_contexts_global_intercept(
    bidi_session,
    url,
    top_context,
    add_intercept,
    fetch,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe
):
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=other_context["context"], url=url(PAGE_EMPTY_HTML), wait="complete"
    )

    # Subscribe to network events in both contexts.
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ],
        contexts=[top_context["context"], other_context["context"]],
    )

    # Add an intercept for top_context and a global intercept.
    text_url = url(PAGE_EMPTY_TEXT)
    context_intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[top_context["context"]],
    )
    global_intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    # Request on the top_context should be blocked and list both intercepts.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=top_context))
    event = await wait_for_future_safe(on_network_event)
    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[context_intercept, global_intercept]
    )

    # Request on the other_context should be blocked by the global intercept.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=other_context))
    event = await wait_for_future_safe(on_network_event)
    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[global_intercept]
    )
