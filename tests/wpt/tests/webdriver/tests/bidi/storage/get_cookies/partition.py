import pytest

from webdriver.bidi.modules.network import NetworkStringValue
from webdriver.bidi.modules.storage import (
    BrowsingContextPartitionDescriptor,
    StorageKeyPartitionDescriptor,
)

from .. import create_cookie
from ... import recursive_compare

pytestmark = pytest.mark.asyncio


async def test_default_partition(
    bidi_session,
    top_context,
    new_tab,
    test_page,
    test_page_cross_origin,
    domain_value,
    add_cookie,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_cross_origin, wait="complete"
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "foo"
    cookie1_value = "bar"
    await add_cookie(new_tab["context"], cookie1_name, cookie1_value)

    cookie2_name = "foo_2"
    cookie2_value = "bar_2"
    await add_cookie(top_context["context"], cookie2_name, cookie2_value)

    cookies = await bidi_session.storage.get_cookies()

    assert cookies["partitionKey"] == {}
    assert len(cookies["cookies"]) == 2
    # Provide consistent cookies order.
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["domain"])
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie1_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie1_value},
        },
        cookie_2,
    )
    recursive_compare(
        {
            "domain": domain_value("alt"),
            "httpOnly": False,
            "name": cookie2_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 10,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_1,
    )


async def test_partition_context(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    add_cookie,
    create_user_context,
    test_page_cross_origin,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    user_context = await create_user_context()
    # Create a new browsing context in another user context.
    new_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    await bidi_session.browsing_context.navigate(
        context=new_context["context"], url=test_page_cross_origin, wait="complete"
    )

    cookie_name = "foo"
    cookie_value = "bar"
    await add_cookie(new_tab["context"], cookie_name, cookie_value)

    # Check that added cookies are present on the right context.
    cookies = await bidi_session.storage.get_cookies(
        partition=BrowsingContextPartitionDescriptor(new_tab["context"])
    )

    # `partitionKey` here might contain `sourceOrigin` for certain browser implementation,
    # so use `recursive_compare` to allow additional fields to be present.
    recursive_compare({"partitionKey": {}}, cookies)

    assert len(cookies["cookies"]) == 1
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie_value},
        },
        cookies["cookies"][0],
    )

    # Check that added cookies are not present on the context in the other user context.
    cookies = await bidi_session.storage.get_cookies(
        partition=BrowsingContextPartitionDescriptor(new_context["context"])
    )

    # `partitionKey` here might contain `sourceOrigin` for certain browser implementation,
    # so use `recursive_compare` to allow additional fields to be present.
    recursive_compare({"partitionKey": {}}, cookies)
    assert len(cookies["cookies"]) == 0


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_partition_context_iframe(
    bidi_session, new_tab, inline, domain_value, domain, set_cookie
):
    iframe_url = inline("<div id='in-iframe'>foo</div>", domain=domain)
    page_url = inline(f"<iframe src='{iframe_url}'></iframe>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]

    cookie_name = "foo"
    cookie_value = "bar"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(domain),
            name=cookie_name,
            value=NetworkStringValue(cookie_value),
        ),
        partition=BrowsingContextPartitionDescriptor(iframe_context["context"]),
    )

    # Check that added cookies are present on the right context
    cookies = await bidi_session.storage.get_cookies(
        partition=BrowsingContextPartitionDescriptor(iframe_context["context"])
    )

    recursive_compare(
        {
            "cookies": [
                {
                    "domain": domain_value(domain=domain),
                    "httpOnly": False,
                    "name": cookie_name,
                    "path": "/",
                    "sameSite": "none",
                    "secure": True,
                    "size": 6,
                    "value": {"type": "string", "value": cookie_value},
                }
            ],
            "partitionKey": {},
        },
        cookies,
    )


@pytest.mark.parametrize(
    "protocol",
    [
        "http",
        "https",
    ],
)
async def test_partition_source_origin(
    bidi_session,
    new_tab,
    top_context,
    inline,
    test_page_cross_origin,
    domain_value,
    origin,
    set_cookie,
    protocol,
):
    url = inline("<div>bar</div>", protocol=protocol)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )
    source_origin_1 = origin(protocol)

    cookie_name = "foo"
    cookie_value = "bar"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            name=cookie_name,
            value=NetworkStringValue(cookie_value),
        ),
        partition=StorageKeyPartitionDescriptor(source_origin=source_origin_1),
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_cross_origin, wait="complete"
    )
    source_origin_2 = origin(domain="alt")

    # Check that added cookies are present on the right origin
    cookies = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(source_origin=source_origin_1)
    )

    recursive_compare(
        {
            "cookies": [
                {
                    "domain": domain_value(),
                    "httpOnly": False,
                    "name": cookie_name,
                    "path": "/",
                    "sameSite": "none",
                    "secure": True,
                    "size": 6,
                    "value": {"type": "string", "value": cookie_value},
                }
            ],
            "partitionKey": {"sourceOrigin": source_origin_1},
        },
        cookies,
    )

    # Check that added cookies are present on the other origin.
    cookies = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(source_origin=source_origin_2)
    )

    recursive_compare(
        {
            "cookies": [],
            "partitionKey": {"sourceOrigin": source_origin_2},
        },
        cookies,
    )
