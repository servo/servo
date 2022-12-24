import asyncio
import json

import pytest

from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

from .. import assert_before_request_sent_event

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"
PAGE_REDIRECT_HTTP_EQUIV = "/webdriver/tests/bidi/network/support/redirect_http_equiv.html"
PAGE_REDIRECTED_HTML = "/webdriver/tests/bidi/network/support/redirected.html"

# The following tests are marked as tentative until
# https://github.com/w3c/webdriver-bidi/pull/204 is merged.


@pytest.fixture
def fetch(bidi_session, top_context):
    """Perform a fetch from the page of the top level context."""
    async def fetch(url, method="GET", headers=None):
        method_arg = f"method: '{method}',"

        headers_arg = ""
        if headers != None:
            headers_arg = f"headers: {json.dumps(headers)},"

        await bidi_session.script.evaluate(
            expression=f"""
               fetch("{url}", {{
                 {method_arg}
                 {headers_arg}
               }})""",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )

    return fetch


@pytest.fixture
async def setup_network_test(bidi_session, subscribe_events, top_context, url):
    """Navigate the current top level context to the provided url and subscribe
    to network.beforeRequestSent.

    Returns an `events` list in which the captured network events will be added.
    """
    remove_listener = None

    async def _setup_network_test(test_url=url(PAGE_EMPTY_HTML)):
        nonlocal remove_listener

        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=test_url,
            wait="complete",
        )
        events = []
        await subscribe_events(["network.beforeRequestSent"])

        async def on_event(method, data):
            events.append(data)

        remove_listener = bidi_session.add_event_listener(
            "network.beforeRequestSent", on_event
        )

        return events

    yield _setup_network_test

    # cleanup
    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_status(bidi_session, top_context, wait_for_event, url, fetch):
    await bidi_session.session.subscribe(events=["network.beforeRequestSent"])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url("/webdriver/tests/bidi/network/support/empty.html"),
        wait="complete",
    )

    # Track all received network.beforeRequestSent events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        "network.beforeRequestSent", on_event
    )

    text_url = url(PAGE_EMPTY_TEXT)
    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url)
    await on_before_request_sent

    assert len(events) == 1
    assert_before_request_sent_event(
        events[0], url=text_url, method="GET", redirect_count=0, is_redirect=False
    )

    await bidi_session.session.unsubscribe(events=["network.beforeRequestSent"])

    # Fetch the text url again, with an additional parameter to bypass the cache
    # check no new event is received.
    await fetch(f"{text_url}?nocache")
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_load_page_twice(
    bidi_session, top_context, wait_for_event, url, fetch, setup_network_test
):
    html_url = url(PAGE_EMPTY_HTML)

    events = await setup_network_test()

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await on_before_request_sent

    assert len(events) == 1
    assert_before_request_sent_event(
        events[0], url=html_url, method="GET", redirect_count=0, is_redirect=False
    )


@pytest.mark.parametrize(
    "method",
    [
        "GET",
        "HEAD",
        "POST",
        "PUT",
        "DELETE",
        "OPTIONS",
        "PATCH",
    ],
)
@pytest.mark.asyncio
async def test_request_method(
    bidi_session, wait_for_event, url, fetch, setup_network_test, method
):
    text_url = url(PAGE_EMPTY_TEXT)

    events = await setup_network_test()

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method=method)
    await on_before_request_sent

    assert len(events) == 1
    assert_before_request_sent_event(
        events[0], url=text_url, method=method, redirect_count=0, is_redirect=False
    )


@pytest.mark.asyncio
async def test_request_headers(
    bidi_session, wait_for_event, url, fetch, setup_network_test
):
    text_url = url(PAGE_EMPTY_TEXT)

    events = await setup_network_test()

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET", headers={"foo": "bar"})
    await on_before_request_sent

    assert len(events) == 1
    assert_before_request_sent_event(
        events[0],
        url=text_url,
        method="GET",
        redirect_count=0,
        is_redirect=False,
        headers=({"name": "foo", "value": "bar"},),
    )


@pytest.mark.asyncio
async def test_request_cookies(
    bidi_session, top_context, wait_for_event, url, fetch, setup_network_test
):
    text_url = url(PAGE_EMPTY_TEXT)

    events = await setup_network_test()

    await bidi_session.script.evaluate(
        expression="document.cookie = 'foo=bar';",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET")
    await on_before_request_sent

    assert len(events) == 1
    assert_before_request_sent_event(
        events[0],
        url=text_url,
        method="GET",
        redirect_count=0,
        is_redirect=False,
        cookies=({"name": "foo", "value": "bar"},),
    )

    await bidi_session.script.evaluate(
        expression="document.cookie = 'fuu=baz';",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET")
    await on_before_request_sent

    assert len(events) == 2
    assert_before_request_sent_event(
        events[1],
        url=text_url,
        method="GET",
        redirect_count=0,
        is_redirect=False,
        cookies=(
            {"name": "foo", "value": "bar"},
            {"name": "fuu", "value": "baz"},
        ),
    )


@pytest.mark.asyncio
async def test_redirect(
    bidi_session, wait_for_event, url, fetch, setup_network_test
):
    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(f"/webdriver/tests/support/redirect.py?location={text_url}")

    events = await setup_network_test()

    await fetch(redirect_url, method="GET")

    # Wait until we receive two events, one for the initial request and one for
    # the redirection.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    assert_before_request_sent_event(
        events[0],
        url=redirect_url,
        method="GET",
        redirect_count=0,
        is_redirect=False,
    )
    assert_before_request_sent_event(
        events[1],
        url=text_url,
        method="GET",
        redirect_count=1,
        is_redirect=True,
    )

    # Check that both requests share the same requestId
    assert events[0]["request"]["request"] == events[1]["request"]["request"]


@pytest.mark.asyncio
async def test_redirect_http_equiv(
    bidi_session, top_context, wait_for_event, url, setup_network_test
):
    # PAGE_REDIRECT_HTTP_EQUIV should redirect to PAGE_REDIRECTED_HTML immediately
    http_equiv_url = url(PAGE_REDIRECT_HTTP_EQUIV)
    redirected_url = url(PAGE_REDIRECTED_HTML)

    events = await setup_network_test()

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=http_equiv_url,
        wait="complete",
    )

    # Wait until we receive two events, one for the initial request and one for
    # the http-equiv "redirect".
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    assert_before_request_sent_event(
        events[0],
        url=http_equiv_url,
        method="GET",
        redirect_count=0,
        is_redirect=False,
    )
    # http-equiv redirect should not be considered as a redirect: redirect_count
    # should be 0 and is_redirect should be false.
    assert_before_request_sent_event(
        events[1],
        url=redirected_url,
        method="GET",
        redirect_count=0,
        is_redirect=False,
    )

    # Check that the http-equiv redirect request has a different requestId
    assert events[0]["request"]["request"] != events[1]["request"]["request"]
