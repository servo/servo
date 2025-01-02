import os
import pytest
from urllib.parse import urlparse

from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.modules.storage import (
    BrowsingContextPartitionDescriptor,
    StorageKeyPartitionDescriptor,
)

from .. import assert_cookies

pytestmark = pytest.mark.asyncio

BEFORE_REQUEST_SENT_EVENT = "network.beforeRequestSent"
PNG_BLACK_DOT = "/webdriver/tests/bidi/storage/get_cookies/support/black_dot.png"


async def test_top_context(
    bidi_session,
    new_tab,
    inline,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
):
    cookie_name = "foo"
    cookie_value = "bar"
    url = inline(
        "<div>with cookies</div>",
        parameters={"pipe": f"header(Set-Cookie, {cookie_name}={cookie_value})"},
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    network_events = await setup_network_test(events=[BEFORE_REQUEST_SENT_EVENT])
    events = network_events[BEFORE_REQUEST_SENT_EVENT]
    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)

    await bidi_session.browsing_context.reload(
        context=new_tab["context"], wait="complete"
    )

    await wait_for_future_safe(on_before_request_sent)

    result = await bidi_session.storage.get_cookies(
        partition=BrowsingContextPartitionDescriptor(new_tab["context"])
    )

    assert_cookies(result["cookies"], events[0]["request"]["cookies"])

    await bidi_session.storage.delete_cookies()


@pytest.mark.parametrize("domain_1", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    new_tab,
    inline,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    domain_1,
):
    cookie_name = "bar"
    cookie_value = "foo"
    iframe_url = inline(
        "<div id='in-iframe'>with cookies</div>",
        domain=domain_1,
        parameters={"pipe": f"header(Set-Cookie, {cookie_name}={cookie_value})"},
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=iframe_url, wait="complete"
    )

    network_events = await setup_network_test(events=[BEFORE_REQUEST_SENT_EVENT])
    events = network_events[BEFORE_REQUEST_SENT_EVENT]
    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)

    page_url = inline(f"<iframe src='{iframe_url}'></iframe>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    await wait_for_future_safe(on_before_request_sent)

    all_contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = all_contexts[0]["children"][0]["context"]

    result = await bidi_session.storage.get_cookies(
        partition=BrowsingContextPartitionDescriptor(iframe_context)
    )

    # Find the network event which belongs to the iframe.
    event_for_iframe = next(
        event for event in events if event["context"] == iframe_context
    )

    assert_cookies(result["cookies"], event_for_iframe["request"]["cookies"])

    # Remove the coookie.
    await bidi_session.storage.delete_cookies()


@pytest.mark.parametrize("domain_1", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_fetch(
    bidi_session,
    new_tab,
    setup_network_test,
    wait_for_event,
    fetch,
    wait_for_future_safe,
    url,
    origin,
    domain_1,
):
    # Clean up cookies in case some other tests failed before cleaning up.
    await bidi_session.storage.delete_cookies()

    # Navigate away from about:blank to make sure document.cookies can be used.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url("/webdriver/tests/bidi/network/support/empty.html"),
        wait="complete",
    )

    cookie_name = "foo"
    cookie_value = "bar"
    path = "/webdriver/tests/support/http_handlers"
    # Add `Access-Control-Allow-Origin` header for cross-origin request to work.
    request_url = url(
        f"{path}/headers.py?header=Access-Control-Allow-Origin:*",
        domain=domain_1,
    )

    await bidi_session.script.evaluate(
        expression=f"document.cookie = '{cookie_name}={cookie_value}';",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    network_events = await setup_network_test(events=[BEFORE_REQUEST_SENT_EVENT])
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await fetch(request_url, method="GET")
    await wait_for_future_safe(on_before_request_sent)

    result = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(source_origin=origin(domain=domain_1)),
        filter={"path": path},
    )
    assert_cookies(result["cookies"], events[0]["request"]["cookies"])

    # Remove the coookie.
    await bidi_session.storage.delete_cookies()


@pytest.mark.parametrize("domain_1", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_image(
    bidi_session,
    new_tab,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    url,
    inline,
    origin,
    domain_1,
):
    # Clean up cookies in case some other tests failed before cleaning up.
    await bidi_session.storage.delete_cookies()

    cookie_name = "bar"
    cookie_value = "foo"

    image_url = url(PNG_BLACK_DOT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=image_url, wait="complete"
    )

    await bidi_session.script.evaluate(
        expression=f"document.cookie = '{cookie_name}={cookie_value}';",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    network_events = await setup_network_test(events=[BEFORE_REQUEST_SENT_EVENT])
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    page_with_image = inline(f"<img src='{image_url}'>", domain=domain_1)

    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_with_image, wait="complete"
    )
    await wait_for_future_safe(on_before_request_sent)

    image_path = os.path.dirname(urlparse(image_url).path)
    result = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(source_origin=origin(domain=domain_1)),
        filter={"path": image_path},
    )

    # Find the network event which belongs to the image.
    event_for_image = next(
        event for event in events if event["request"]["url"] == image_url
    )
    assert_cookies(result["cookies"], event_for_image["request"]["cookies"])

    # Remove the coookie.
    await bidi_session.storage.delete_cookies()
