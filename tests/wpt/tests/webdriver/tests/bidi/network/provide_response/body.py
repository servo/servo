import pytest

from webdriver.bidi.modules.network import NetworkStringValue
from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

from .. import (
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
    PAGE_PROVIDE_RESPONSE_HTML,
    PAGE_PROVIDE_RESPONSE_SCRIPT,
    PAGE_PROVIDE_RESPONSE_STYLESHEET,
)

pytestmark = pytest.mark.asyncio

LOAD_EVENT = "browsingContext.load"


@pytest.mark.parametrize(
    "blocked_url, body, expression",
    [
        (
            PAGE_PROVIDE_RESPONSE_HTML,
            "<div id=from-provide-response>",
            "!!document.getElementById('from-provide-response')",
        ),
        (
            PAGE_PROVIDE_RESPONSE_SCRIPT,
            "window.isFromProvideResponse = true;",
            "window.isFromProvideResponse == true;",
        ),
        (
            PAGE_PROVIDE_RESPONSE_STYLESHEET,
            "div { color: rgb(255, 0, 0) }",
            """
              const div = document.querySelector('div');
              window.getComputedStyle(div).color === 'rgb(255, 0, 0)'
            """,
        ),
    ],
)
async def test_body_before_request_sent(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
    blocked_url,
    body,
    expression,
):
    request = await setup_blocked_request(
        phase="beforeRequestSent",
        navigate=True,
        blocked_url=url(blocked_url),
        navigate_url=url(PAGE_PROVIDE_RESPONSE_HTML),
    )

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
            LOAD_EVENT,
        ]
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)

    on_load = wait_for_event(LOAD_EVENT)

    await bidi_session.network.provide_response(
        request=request,
        body=NetworkStringValue(body),
        status_code=200,
        reason_phrase="OK",
    )

    await wait_for_future_safe(on_response_completed)
    await wait_for_future_safe(on_load)

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result["value"] is True
