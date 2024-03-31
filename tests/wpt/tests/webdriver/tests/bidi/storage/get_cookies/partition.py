import pytest

from webdriver.bidi.modules.network import NetworkStringValue
from webdriver.bidi.modules.storage import (
    BrowsingContextPartitionDescriptor,
    StorageKeyPartitionDescriptor,
)

from .. import create_cookie, get_default_partition_key
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

    assert cookies["partitionKey"] == {
        **(await get_default_partition_key(bidi_session)),
    }
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

    assert cookies["partitionKey"] == {
        **(await get_default_partition_key(bidi_session, new_tab["context"])),
        "userContext": "default"
    }
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

    assert cookies["partitionKey"] == {
        **(await get_default_partition_key(bidi_session, new_context["context"])),
        "userContext": user_context
    }
    assert len(cookies["cookies"]) == 0


async def test_partition_context_with_different_domain(
    bidi_session, set_cookie, new_tab, test_page, domain_value
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    # Set cookie on a different domain.
    cookie_domain = domain_value(domain="alt")
    cookie_name = "foo"
    cookie_value = "bar"
    partition = BrowsingContextPartitionDescriptor(new_tab["context"])
    await set_cookie(
        cookie=create_cookie(
            domain=cookie_domain,
            name=cookie_name,
            value=NetworkStringValue(cookie_value),
        ),
        partition=partition,
    )

    result = await bidi_session.storage.get_cookies(
        partition=BrowsingContextPartitionDescriptor(new_tab["context"])
    )

    recursive_compare([
        {
            "domain": cookie_domain,
            "httpOnly": False,
            "name": cookie_name,
            "path": "/",
            "sameSite": "none",
            "secure": True,
            "size": 6,
            "value": {"type": "string", "value": cookie_value},
        }
    ], result["cookies"])


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
            "partitionKey": {"userContext": "default"},
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


async def test_partition_default_user_context(
    bidi_session,
    test_page,
    domain_value,
    add_cookie,
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=new_context["context"], url=test_page, wait="complete"
    )

    cookie_name = "foo"
    cookie_value = "bar"
    await add_cookie(new_context["context"], cookie_name, cookie_value)

    # Check that added cookies are present on the right user context.
    result = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(user_context="default")
    )
    expected_cookies = [
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie_value},
        }
    ]
    recursive_compare(
        {
            "cookies": expected_cookies,
            "partitionKey": {"userContext": "default"},
        },
        result,
    )


async def test_partition_user_context(
    bidi_session,
    test_page,
    domain_value,
    create_user_context,
    test_page_cross_origin,
    add_cookie,
):
    user_context_1 = await create_user_context()
    new_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    await bidi_session.browsing_context.navigate(
        context=new_context_1["context"], url=test_page, wait="complete"
    )

    user_context_2 = await create_user_context()
    new_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )
    await bidi_session.browsing_context.navigate(
        context=new_context_2["context"], url=test_page_cross_origin, wait="complete"
    )

    cookie_name = "foo_1"
    cookie_value = "bar_1"
    await add_cookie(new_context_1["context"], cookie_name, cookie_value)

    # Check that added cookies are present on the right user context.
    result = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(user_context=user_context_1)
    )
    expected_cookies = [
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 10,
            "value": {"type": "string", "value": cookie_value},
        }
    ]
    recursive_compare(
        {
            "cookies": expected_cookies,
            "partitionKey": {"userContext": user_context_1},
        },
        result,
    )

    # Check that added cookies are not present on the other user context.
    result = await bidi_session.storage.get_cookies(
        partition=StorageKeyPartitionDescriptor(user_context=user_context_2)
    )

    recursive_compare(
        {
            "cookies": [],
            "partitionKey": {"userContext": user_context_2},
        },
        result,
    )
