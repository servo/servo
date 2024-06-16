import pytest

from webdriver.bidi.modules.network import Header, NetworkStringValue
from webdriver.bidi.modules.script import ContextTarget

from .. import assert_response_event, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "request_headers, modified_headers",
    [
        [{"a": "1"}, {}],
        [{}, {"b": "2"}],
        [{"a": "1", "b": "2"}, {"c": "3", "d": "4"}],
        [{"a": "1"}, {"a": "not-1"}],
    ],
)
async def test_modify_headers(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
    request_headers,
    modified_headers,
):
    request = await setup_blocked_request("beforeRequestSent", headers=request_headers)
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    headers = []
    for name, value in modified_headers.items():
        headers.append(Header(name=name, value=NetworkStringValue(value)))

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(request=request, headers=headers)
    response_event = await on_response_completed
    assert_response_event(response_event, expected_request={"headers": headers})


async def test_override_cookies(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
    top_context,
    url
):
    # Navigate away from about:blank to make sure document.cookies can be used.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url("/webdriver/tests/bidi/support/empty.html"),
        wait="complete"
    )

    await bidi_session.script.evaluate(
        expression="document.cookie = 'foo=bar';",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    request = await setup_blocked_request("beforeRequestSent")
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(request=request, headers=[])
    response_event = await on_response_completed
    assert len(response_event["request"]["cookies"]) == 0
