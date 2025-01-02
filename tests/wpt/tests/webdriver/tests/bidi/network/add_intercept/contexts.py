import asyncio

import pytest
from webdriver.bidi.modules.script import ScriptEvaluateResultException

from .. import (
    assert_before_request_sent_event,
    assert_response_event,
    PAGE_EMPTY_HTML,
    PAGE_EMPTY_TEXT,
    BEFORE_REQUEST_SENT_EVENT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
    PHASE_TO_EVENT_MAP,
)


@pytest.mark.asyncio
@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("phase", ["beforeRequestSent", "responseStarted"])
async def test_frame_context(
    bidi_session,
    url,
    inline,
    new_tab,
    add_intercept,
    fetch,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    domain,
    phase
):
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[new_tab["context"]],
    )

    frame_url = inline("<div>foo</div>")
    test_url = inline(f"<iframe src='{frame_url}'></iframe>", domain=domain)
    await bidi_session.browsing_context.navigate(
        url=test_url, context=new_tab["context"], wait="complete"
    )

    # Retrieve the context for the iframe.
    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]

    # Add an intercept.
    text_url = url(PAGE_EMPTY_TEXT)
    await add_intercept(
        phases=[phase],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[new_tab["context"]],
    )

    # Request in the iframe context should be blocked.
    [event_name, assert_network_event] = PHASE_TO_EVENT_MAP[phase]
    on_network_event = wait_for_event(event_name)
    asyncio.ensure_future(fetch(text_url, context=frame))
    event = await wait_for_future_safe(on_network_event)
    assert_network_event(event, is_blocked=True)


@pytest.mark.asyncio
@pytest.mark.parametrize("phase", ["beforeRequestSent", "responseStarted"])
async def test_other_context(
    bidi_session,
    url,
    new_tab,
    add_intercept,
    fetch,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    phase
):
    # Subscribe to network events only in new_tab
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        contexts=[new_tab["context"]],
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


    # Request to new_tab should be blocked.
    [event_name, assert_network_event] = PHASE_TO_EVENT_MAP[phase]
    on_network_event = wait_for_event(event_name)
    asyncio.ensure_future(fetch(text_url, context=new_tab))
    event = await wait_for_future_safe(on_network_event)
    assert_network_event(event, is_blocked=True)

    # Request to other_context should not be blocked because we are not
    # subscribed to network events. Wait for fetch to resolve successfully.
    await asyncio.ensure_future(fetch(text_url, context=other_context))


@pytest.mark.asyncio
async def test_other_context_with_event_subscription(
    bidi_session,
    url,
    new_tab,
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
        contexts=[new_tab["context"], other_context["context"]],
    )

    # Add an intercept to new_tab only.
    text_url = url(PAGE_EMPTY_TEXT)
    await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[new_tab["context"]]
    )

    # Request to the new_tab should be blocked.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=new_tab))
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
    new_tab,
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
        contexts=[new_tab["context"], other_context["context"]],
    )

    # Add an intercept to both contexts
    text_url = url(PAGE_EMPTY_TEXT)
    intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[new_tab["context"], other_context["context"]],
    )

    # Request on the new_tab should be blocked.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=new_tab))
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
    new_tab,
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
        contexts=[new_tab["context"], other_context["context"]],
    )

    # Add an intercept for new_tab and a global intercept.
    text_url = url(PAGE_EMPTY_TEXT)
    context_intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
        contexts=[new_tab["context"]],
    )
    global_intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    # Request on the new_tab should be blocked and list both intercepts.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url, context=new_tab))
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
