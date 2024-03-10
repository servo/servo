import pytest

from webdriver.bidi.modules.network import NetworkStringValue
from webdriver.bidi.modules.storage import (
    BrowsingContextPartitionDescriptor,
    PartialCookie,
    StorageKeyPartitionDescriptor,
)

from . import assert_cookies_are_not_present
from .. import assert_cookie_is_set, create_cookie, get_default_partition_key
from ... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "with_document_cookie",
    [True, False],
    ids=["with document cookie", "with set cookie"],
)
async def test_default_partition(
    bidi_session,
    top_context,
    new_tab,
    test_page,
    test_page_cross_origin,
    domain_value,
    add_cookie,
    set_cookie,
    with_document_cookie,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_cross_origin, wait="complete"
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "foo"
    cookie1_value = "bar"

    cookie2_name = "foo_2"
    cookie2_value = "bar_2"

    if with_document_cookie:
        await add_cookie(new_tab["context"], cookie1_name, cookie1_value)
        await add_cookie(top_context["context"], cookie2_name, cookie2_value)
    else:
        await set_cookie(
            cookie=create_cookie(
                domain=domain_value("alt"),
                name=cookie1_name,
                value=NetworkStringValue(cookie1_value),
            )
        )
        await set_cookie(
            cookie=create_cookie(
                domain=domain_value(),
                name=cookie2_name,
                value=NetworkStringValue(cookie2_value),
            )
        )

    result = await bidi_session.storage.delete_cookies()
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    await assert_cookies_are_not_present(bidi_session)


@pytest.mark.parametrize(
    "with_document_cookie",
    [True, False],
    ids=["with document cookie", "with set cookie"],
)
async def test_partition_context(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    add_cookie,
    set_cookie,
    with_document_cookie,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie_name = "foo"
    cookie_value = "bar"
    partition = BrowsingContextPartitionDescriptor(new_tab["context"])
    if with_document_cookie:
        await add_cookie(new_tab["context"], cookie_name, cookie_value)
    else:
        await set_cookie(
            cookie=create_cookie(
                domain=domain_value(),
                name=cookie_name,
                value=NetworkStringValue(cookie_value),
            ),
            partition=partition,
        )

    result = await bidi_session.storage.delete_cookies(partition=partition)
    assert result == {"partitionKey": (await get_default_partition_key(bidi_session, new_tab["context"]))}

    await assert_cookies_are_not_present(bidi_session, partition)


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
    frame_partition = BrowsingContextPartitionDescriptor(iframe_context["context"])
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(domain),
            name=cookie_name,
            value=NetworkStringValue(cookie_value),
        ),
        partition=frame_partition,
    )

    result = await bidi_session.storage.delete_cookies(partition=frame_partition)
    assert result == {"partitionKey": (await get_default_partition_key(bidi_session, new_tab["context"]))}

    await assert_cookies_are_not_present(bidi_session, frame_partition)


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
    test_page,
    inline,
    domain_value,
    origin,
    set_cookie,
    protocol,
):
    url = inline("<div>bar</div>", protocol=protocol, domain="alt")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_domain = domain_value(domain="alt")
    cookie1_name = "foo"
    cookie1_value = "bar"
    cookie1_source_origin = origin(domain="alt", protocol=protocol)
    cookie1_partition = StorageKeyPartitionDescriptor(
        source_origin=cookie1_source_origin
    )
    await set_cookie(
        cookie=create_cookie(
            domain=cookie1_domain,
            name=cookie1_name,
            value=NetworkStringValue(cookie1_value),
        ),
        partition=cookie1_partition,
    )

    cookie2_domain = domain_value()
    cookie2_name = "bar"
    cookie2_value = "foo"
    cookie2_source_origin = origin()
    cookie2_partition = StorageKeyPartitionDescriptor(
        source_origin=cookie2_source_origin
    )
    await set_cookie(
        cookie=create_cookie(
            domain=cookie2_domain,
            name=cookie2_name,
            value=NetworkStringValue(cookie2_value),
        ),
        partition=cookie2_partition,
    )

    result = await bidi_session.storage.delete_cookies(partition=cookie1_partition)
    assert result == {
        "partitionKey": {
            **(await get_default_partition_key(bidi_session)),
            "sourceOrigin": cookie1_source_origin
        }
    }

    await assert_cookies_are_not_present(bidi_session, partition=cookie1_partition)

    # Check that the second cookie is still present on another origin.
    await assert_cookie_is_set(
        bidi_session=bidi_session,
        domain=domain_value(),
        name=cookie2_name,
        value={"type": "string", "value": cookie2_value},
        partition=cookie2_partition,
    )


@pytest.mark.parametrize(
    "with_document_cookie",
    [True, False],
    ids=["with document cookie", "with set cookie"],
)
async def test_partition_user_context(
    bidi_session,
    test_page,
    create_user_context,
    test_page_cross_origin,
    domain_value,
    add_cookie,
    set_cookie,
    with_document_cookie,
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

    cookie1_domain = domain_value()
    cookie1_name = "foo_1"
    cookie1_value = "bar_1"
    cookie1_partition = StorageKeyPartitionDescriptor(user_context=user_context_1)

    cookie2_domain = domain_value(domain="alt")
    cookie2_name = "foo_2"
    cookie2_value = "bar_2"
    cookie2_partition = StorageKeyPartitionDescriptor(user_context=user_context_2)

    if with_document_cookie:
        await add_cookie(
            new_context_1["context"], cookie1_name, cookie1_value, path="/"
        )
        await add_cookie(
            new_context_2["context"], cookie2_name, cookie2_value, path="/"
        )
    else:
        await set_cookie(
            cookie=PartialCookie(
                domain=cookie1_domain,
                name=cookie1_name,
                value=NetworkStringValue(cookie1_value),
            ),
            partition=cookie1_partition,
        )
        await set_cookie(
            cookie=PartialCookie(
                domain=cookie2_domain,
                name=cookie2_name,
                value=NetworkStringValue(cookie2_value),
            ),
            partition=cookie2_partition,
        )

    result = await bidi_session.storage.delete_cookies(
        partition=StorageKeyPartitionDescriptor(user_context=user_context_1)
    )
    assert result == {
        "partitionKey": {
            **(await get_default_partition_key(bidi_session)),
            "userContext": user_context_1
        }
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, partition=cookie1_partition)

    # Check that the second cookie is still present on another origin.
    await assert_cookie_is_set(
        bidi_session=bidi_session,
        domain=cookie2_domain,
        name=cookie2_name,
        value={"type": "string", "value": cookie2_value},
        partition=cookie2_partition,
        secure=False
    )
