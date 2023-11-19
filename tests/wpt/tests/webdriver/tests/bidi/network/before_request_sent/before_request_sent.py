import asyncio

import pytest

from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

from .. import assert_before_request_sent_event

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"
PAGE_REDIRECT_HTTP_EQUIV = (
    "/webdriver/tests/bidi/network/support/redirect_http_equiv.html"
)
PAGE_REDIRECTED_HTML = "/webdriver/tests/bidi/network/support/redirected.html"


@pytest.mark.asyncio
async def test_subscribe_status(bidi_session, subscribe_events, top_context, wait_for_event, wait_for_future_safe, url, fetch):
    await subscribe_events(events=["network.beforeRequestSent"])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url(PAGE_EMPTY_HTML),
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
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": text_url}
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )

    await bidi_session.session.unsubscribe(events=["network.beforeRequestSent"])

    # Fetch the text url again, with an additional parameter to bypass the cache
    # and check no new event is received.
    await fetch(f"{text_url}?nocache")
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_load_page_twice(
    bidi_session, top_context, wait_for_event, url, setup_network_test, wait_for_future_safe
):
    html_url = url(PAGE_EMPTY_HTML)

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": html_url}
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_navigation_id(
    bidi_session, top_context, wait_for_event, url, fetch, setup_network_test, wait_for_future_safe
):
    html_url = url(PAGE_EMPTY_HTML)

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": html_url}
    assert_before_request_sent_event(
        events[0], expected_request=expected_request, navigation=result["navigation"]
    )
    assert events[0]["navigation"] is not None

    text_url = url(PAGE_EMPTY_TEXT)
    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET")
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 2
    expected_request = {"method": "GET", "url": text_url}
    assert_before_request_sent_event(
        events[1],
        expected_request=expected_request,
    )
    # Check that requests not related to a navigation have no navigation id.
    assert events[1]["navigation"] is None


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
    wait_for_event, wait_for_future_safe, url, fetch, setup_network_test, method
):
    text_url = url(PAGE_EMPTY_TEXT)

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method=method)
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {"method": method, "url": text_url}
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_request_headers(
    wait_for_event, wait_for_future_safe, url, fetch, setup_network_test
):
    text_url = url(PAGE_EMPTY_TEXT)

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET", headers={"foo": "bar"})
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {
        "headers": ({"name": "foo", "value": {"type": "string", "value": "bar"}},),
        "method": "GET",
        "url": text_url,
    }
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_request_cookies(
    bidi_session, top_context, wait_for_event, wait_for_future_safe, url, fetch, setup_network_test
):
    text_url = url(PAGE_EMPTY_TEXT)

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    await bidi_session.script.evaluate(
        expression="document.cookie = 'foo=bar';",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET")
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {
        "cookies": ({"name": "foo", "value": {"type": "string", "value": "bar"}},),
        "method": "GET",
        "url": text_url,
    }
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )

    await bidi_session.script.evaluate(
        expression="document.cookie = 'fuu=baz';",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    on_before_request_sent = wait_for_event("network.beforeRequestSent")
    await fetch(text_url, method="GET")
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 2

    expected_request = {
        "cookies": (
            {"name": "foo", "value": {"type": "string", "value": "bar"}},
            {"name": "fuu", "value": {"type": "string", "value": "baz"}},
        ),
        "method": "GET",
        "url": text_url,
    }
    assert_before_request_sent_event(
        events[1],
        expected_request=expected_request,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_redirect(bidi_session, wait_for_event, url, fetch, setup_network_test):
    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={text_url}"
    )

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    await fetch(redirect_url, method="GET")

    # Wait until we receive two events, one for the initial request and one for
    # the redirection.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    expected_request = {"method": "GET", "url": redirect_url}
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )
    expected_request = {"method": "GET", "url": text_url}
    assert_before_request_sent_event(
        events[1], expected_request=expected_request, redirect_count=1
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

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=http_equiv_url,
        wait="complete",
    )

    # Wait until we receive two events, one for the initial request and one for
    # the http-equiv "redirect".
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    expected_request = {"method": "GET", "url": http_equiv_url}
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
        navigation=result["navigation"],
    )
    # http-equiv redirect should not be considered as a redirect: redirect_count
    # should be 0.
    expected_request = {"method": "GET", "url": redirected_url}
    assert_before_request_sent_event(
        events[1],
        expected_request=expected_request,
        redirect_count=0,
    )

    # Check that the http-equiv redirect request has a different requestId
    assert events[0]["request"]["request"] != events[1]["request"]["request"]

    # Check that the http-equiv redirect request also has a navigation id set,
    # but different from the original request.
    assert events[1]["navigation"] is not None
    assert events[1]["navigation"] != events[0]["navigation"]


@pytest.mark.asyncio
async def test_redirect_navigation(
    bidi_session, top_context, wait_for_event, url, setup_network_test
):
    html_url = url(PAGE_EMPTY_HTML)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={html_url}"
    )

    network_events = await setup_network_test(events=["network.beforeRequestSent"])
    events = network_events["network.beforeRequestSent"]

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=redirect_url,
        wait="complete",
    )

    assert len(events) == 2
    expected_request = {"method": "GET", "url": redirect_url}
    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
        navigation=result["navigation"],
        redirect_count=0,
    )
    expected_request = {"method": "GET", "url": html_url}
    assert_before_request_sent_event(
        events[1],
        expected_request=expected_request,
        navigation=result["navigation"],
        redirect_count=1,
    )

    # Check that both requests share the same requestId
    assert events[0]["request"]["request"] == events[1]["request"]["request"]
