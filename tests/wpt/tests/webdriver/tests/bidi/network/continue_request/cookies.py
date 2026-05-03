import pytest
import pytest_asyncio

from webdriver.bidi.modules.network import CookieHeader, Header, NetworkStringValue
from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare
from .. import RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture(autouse=True)
async def delete_cookies(bidi_session):
    await bidi_session.storage.delete_cookies()


@pytest.mark.parametrize(
    "document_cookies, modified_cookies, expected_event_cookies",
    [
        [{"a": "from-store"}, {}, []],
        [{"a": "from-store"}, {"b": "from-command"}, []],
        [{}, {"b": "from-command"}, []],
        [
            {"a": "from-store"},
            {"a": "from-command"},
            [{"name": "a", "value": {"type": "string", "value": "from-store"}}],
        ],
        [
            {"a": "from-store-a", "b": "from-store-b"},
            {"b": "from-command-b", "c": "from-command-c"},
            [{"name": "b", "value": {"type": "string", "value": "from-store-b"}}],
        ],
    ],
    ids=[
        "store_cookie_removed",
        "store_cookies_replaced",
        "new_cookie_in_command",
        "modified_cookie_in_command",
        "common_cookie",
    ],
)
async def test_modify_cookies(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
    top_context,
    document_cookies,
    modified_cookies,
    expected_event_cookies,
    url,
):
    # Navigate away from about:blank to make sure document.cookies can be used.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url("/webdriver/tests/bidi/network/support/empty.html"),
        wait="complete",
    )

    expression = ""
    for name, value in document_cookies.items():
        expression += f"document.cookie = '{name}={value}';"

    await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    request = await setup_blocked_request("beforeRequestSent")
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    cookies = []
    for name, value in modified_cookies.items():
        cookies.append(CookieHeader(name=name, value=NetworkStringValue(value)))

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(request=request, cookies=cookies)
    response_event = await on_response_completed

    event_cookies = response_event["request"]["cookies"]
    assert len(event_cookies) == len(expected_event_cookies)
    for expected in expected_event_cookies:
        event_cookie = next(
            filter(lambda c: c["name"] == expected["name"], event_cookies), None
        )
        recursive_compare(expected, event_cookie)

    await bidi_session.storage.delete_cookies()


async def test_override_header_cookie(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
):
    request = await setup_blocked_request(
        "beforeRequestSent", headers={"Cookie": "a=1"}
    )
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    cookie = CookieHeader(name="b", value=NetworkStringValue("2"))
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(request=request, cookies=[cookie])
    response_event = await on_response_completed

    event_cookies = response_event["request"]["cookies"]
    assert event_cookies == []

    await bidi_session.storage.delete_cookies()


@pytest.mark.parametrize(
    "header_cookie",
    ["a=from-header", "b=from-header"],
    ids=["different_cookie", "same_cookie"],
)
async def test_override_modified_header_cookies(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, header_cookie
):
    request = await setup_blocked_request("beforeRequestSent")
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    header = Header(name="Cookie", value=NetworkStringValue(header_cookie))
    cookie = CookieHeader(name="b", value=NetworkStringValue("2"))
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(
        request=request, headers=[header], cookies=[cookie]
    )
    response_event = await on_response_completed

    event_cookies = response_event["request"]["cookies"]
    assert event_cookies == []

    await bidi_session.storage.delete_cookies()
