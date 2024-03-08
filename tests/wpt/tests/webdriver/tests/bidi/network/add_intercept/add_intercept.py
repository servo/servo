import asyncio
import uuid

import pytest
from webdriver.bidi.modules.script import ScriptEvaluateResultException

from .. import (
    assert_before_request_sent_event,
    PAGE_EMPTY_HTML,
    PAGE_EMPTY_TEXT,
    PAGE_OTHER_TEXT,
    BEFORE_REQUEST_SENT_EVENT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)


@pytest.mark.asyncio
@pytest.mark.parametrize("phase", ["beforeRequestSent", "responseStarted"])
async def test_other_url(
    url,
    add_intercept,
    fetch,
    setup_network_test,
    phase,
):
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
    )

    # Add an intercept.
    text_url = url(PAGE_EMPTY_TEXT)
    await add_intercept(
        phases=[phase],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    # Request to PAGE_EMPTY_TEXT should be blocked and throw a ScriptEvaluateResultException
    # from the AbortController.
    with pytest.raises(ScriptEvaluateResultException):
        await fetch(text_url)

    # Request to PAGE_OTHER_TEXT should not be blocked.
    await fetch(url(PAGE_OTHER_TEXT))


@pytest.mark.asyncio
async def test_return_value(add_intercept):
    intercept = await add_intercept(phases=["beforeRequestSent"], url_patterns=[])

    assert isinstance(intercept, str)
    uuid.UUID(hex=intercept)


@pytest.mark.asyncio
async def test_two_intercepts(
    bidi_session,
    wait_for_event,
    url,
    add_intercept,
    fetch,
    setup_network_test,
    wait_for_future_safe,
):
    await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
    )

    # Add a string intercept to catch requests to PAGE_EMPTY_TEXT.
    text_url = url(PAGE_EMPTY_TEXT)
    string_intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )
    # Add a second intercept to catch all requests.
    global_intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[],
    )

    # Perform a request to PAGE_EMPTY_TEXT, which should match both intercepts
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[string_intercept, global_intercept]
    )

    # Perform a request to PAGE_OTHER_TEXT, which should only match one intercept
    other_url = url(PAGE_OTHER_TEXT)

    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(other_url))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[global_intercept]
    )

    # Remove the global intercept, requests to PAGE_OTHER_TEXT should no longer
    # be blocked.
    await bidi_session.network.remove_intercept(intercept=global_intercept)
    await fetch(other_url)

    # Requests to PAGE_EMPTY_TEXT should still be blocked, but only by one
    # intercept.
    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(text_url))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(
        event, is_blocked=True, intercepts=[string_intercept]
    )

    # Remove the string intercept, requests to PAGE_EMPTY_TEXT should no longer
    # be blocked.
    await bidi_session.network.remove_intercept(intercept=string_intercept)
    await fetch(text_url)
