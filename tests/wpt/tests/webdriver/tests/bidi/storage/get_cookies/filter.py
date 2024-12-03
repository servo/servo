import pytest
from webdriver.bidi.modules.network import NetworkBase64Value, NetworkStringValue
from webdriver.bidi.modules.storage import CookieFilter

from .. import assert_partition_key, create_cookie, format_expiry_string, generate_expiry_date
from ... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "filter",
    [
        {"size": 6},
        {"value": NetworkStringValue("bar")},
        {"value": NetworkBase64Value("YmFy")},
    ],
)
async def test_filter(
    bidi_session, new_tab, test_page, domain_value, add_cookie, filter
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )
    value_1 = "bar"

    cookie1_name = "baz"
    await add_cookie(new_tab["context"], cookie1_name, value_1)

    cookie2_name = "foo"
    await add_cookie(new_tab["context"], cookie2_name, value_1)

    cookie3_name = "foo_3"
    await add_cookie(new_tab["context"], cookie3_name, "bar_3")

    cookies = await bidi_session.storage.get_cookies(
        filter=filter,
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    # Provide consistent cookies order.
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie1_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": value_1},
        },
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie2_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": value_1},
        },
        cookie_2,
    )


async def test_filter_domain(
    bidi_session,
    top_context,
    new_tab,
    test_page,
    test_page_cross_origin,
    domain_value,
    add_cookie,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page, wait="complete"
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page_cross_origin, wait="complete"
    )

    cookie1_name = "bar"
    cookie1_value = "foo"
    await add_cookie(top_context["context"], cookie1_name, cookie1_value)

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(top_context["context"], cookie2_name, cookie2_value)

    cookie3_name = "foo_2"
    cookie3_value = "bar_2"
    await add_cookie(new_tab["context"], cookie3_name, cookie3_value)
    domain = domain_value()

    cookies = await bidi_session.storage.get_cookies(
        filter=CookieFilter(domain=domain),
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    # Provide consistent cookies order.
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
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
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie2_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_2,
    )


@pytest.mark.parametrize(
    "expiry_diff_1, expiry_diff_2",
    [
        (1, 2),
        (1, None),
    ],
)
async def test_filter_expiry(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    add_cookie,
    expiry_diff_1,
    expiry_diff_2,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_expiry_date = generate_expiry_date(expiry_diff_1)
    cookie1_expiry = int(cookie1_expiry_date.timestamp())
    cookie1_date_string = format_expiry_string(cookie1_expiry_date)

    cookie1_name = "bar"
    cookie1_value = "foo"
    await add_cookie(
        context=new_tab["context"],
        name=cookie1_name,
        value=cookie1_value,
        expiry=cookie1_date_string,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        context=new_tab["context"],
        name=cookie2_name,
        value=cookie2_value,
        expiry=cookie1_date_string,
    )

    cookie3_name = "foo_3"
    if expiry_diff_2 is None:
        cookie2_date_string = None
    else:
        cookie2_expiry_date = generate_expiry_date(expiry_diff_2)
        cookie2_date_string = format_expiry_string(cookie2_expiry_date)

    await add_cookie(
        new_tab["context"], cookie3_name, "bar_3", expiry=cookie2_date_string
    )

    cookies = await bidi_session.storage.get_cookies(
        filter=CookieFilter(expiry=cookie1_expiry),
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    # Provide consistent cookies order.
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
    recursive_compare(
        {
            "domain": domain_value(),
            "expiry": cookie1_expiry,
            "httpOnly": False,
            "name": cookie1_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie1_value},
        },
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "expiry": cookie1_expiry,
            "httpOnly": False,
            "name": cookie2_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_2,
    )


async def test_filter_name(bidi_session, new_tab, test_page, domain_value, add_cookie):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "foo"
    cookie1_value = "bar"
    await add_cookie(new_tab["context"], cookie1_name, cookie1_value)

    cookie2_name = "foo_2"
    await add_cookie(new_tab["context"], cookie2_name, "bar_2")

    cookies = await bidi_session.storage.get_cookies(
        filter={"name": "foo"},
    )

    recursive_compare(
        {
            "cookies": [
                {
                    "domain": domain_value(),
                    "httpOnly": False,
                    "name": cookie1_name,
                    "path": "/webdriver/tests/support",
                    "sameSite": "none",
                    "secure": False,
                    "size": 6,
                    "value": {"type": "string", "value": cookie1_value},
                }
            ],
            "partitionKey": {},
        },
        cookies,
    )


@pytest.mark.parametrize(
    "same_site_1, same_site_2",
    [
        ("none", "strict"),
        ("lax", "none"),
        ("strict", "none"),
        ("lax", "strict"),
        ("strict", "lax"),
    ],
)
async def test_filter_same_site(
    bidi_session, new_tab, test_page, domain_value, same_site_1, same_site_2, add_cookie
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "bar"
    cookie1_value = "foo"
    await add_cookie(
        new_tab["context"],
        cookie1_name,
        cookie1_value,
        same_site=same_site_1,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        new_tab["context"],
        cookie2_name,
        cookie2_value,
        same_site=same_site_1,
    )

    cookie3_name = "foo_3"
    await add_cookie(new_tab["context"], cookie3_name, "bar_3", same_site=same_site_2)

    cookies = await bidi_session.storage.get_cookies(
        filter=CookieFilter(same_site=same_site_1),
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    # Provide consistent cookies order.
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie1_name,
            "path": "/webdriver/tests/support",
            "sameSite": same_site_1,
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie1_value},
        },
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie2_name,
            "path": "/webdriver/tests/support",
            "sameSite": same_site_1,
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_2,
    )


@pytest.mark.parametrize(
    "secure_1, secure_2",
    [(True, False), (False, True)],
)
async def test_filter_secure(
    bidi_session, new_tab, test_page, domain_value, add_cookie, secure_1, secure_2
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "bar"
    cookie1_value = "foo"
    await add_cookie(
        new_tab["context"],
        cookie1_name,
        cookie1_value,
        secure=secure_1,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        new_tab["context"],
        cookie2_name,
        cookie2_value,
        secure=secure_1,
    )

    cookie3_name = "foo_3"
    await add_cookie(new_tab["context"], cookie3_name, "bar_3", secure=secure_2)

    cookies = await bidi_session.storage.get_cookies(
        filter=CookieFilter(secure=secure_1),
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    # Provide consistent cookies order.
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie1_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": secure_1,
            "size": 6,
            "value": {"type": "string", "value": cookie1_value},
        },
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie2_name,
            "path": "/webdriver/tests/support",
            "sameSite": "none",
            "secure": secure_1,
            "size": 6,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_2,
    )


@pytest.mark.parametrize(
    "path_1, path_2",
    [
        ("/webdriver/tests/support", "/"),
        ("/", None),
        ("/webdriver", "/webdriver/tests"),
    ],
)
async def test_filter_path(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    add_cookie,
    path_1,
    path_2,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "bar"
    cookie1_value = "foo"
    await add_cookie(
        new_tab["context"],
        cookie1_name,
        cookie1_value,
        path=path_1,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        new_tab["context"],
        cookie2_name,
        cookie2_value,
        path=path_1,
    )

    cookie3_name = "foo_3"
    await add_cookie(new_tab["context"], cookie3_name, "bar_3", path=path_2)

    cookies = await bidi_session.storage.get_cookies(
        filter=CookieFilter(path=path_1),
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie1_name,
            "path": path_1,
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie1_value},
        },
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": False,
            "name": cookie2_name,
            "path": path_1,
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_2,
    )


@pytest.mark.parametrize(
    "http_only_1, http_only_2",
    [(True, False), (False, True)],
)
async def test_filter_http_only(
    bidi_session, new_tab, test_page, domain_value, set_cookie, http_only_1, http_only_2
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    cookie1_name = "bar"
    cookie1_value = "foo"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            name=cookie1_name,
            value=NetworkStringValue(cookie1_value),
            http_only=http_only_1,
        )
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            name=cookie2_name,
            value=NetworkStringValue(cookie2_value),
            http_only=http_only_1,
        )
    )

    cookie3_name = "foo_2"
    cookie3_value = "bar_2"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            name=cookie3_name,
            value=NetworkStringValue(cookie3_value),
            http_only=http_only_2,
        )
    )

    cookies = await bidi_session.storage.get_cookies(
        filter=CookieFilter(http_only=http_only_1),
    )

    await assert_partition_key(bidi_session, actual=cookies["partitionKey"])
    assert len(cookies["cookies"]) == 2
    (cookie_1, cookie_2) = sorted(cookies["cookies"], key=lambda c: c["name"])
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": http_only_1,
            "name": cookie1_name,
            "path": "/",
            "sameSite": "none",
            "secure": True,
            "size": 6,
            "value": {"type": "string", "value": cookie1_value},
        },
        cookie_1,
    )
    recursive_compare(
        {
            "domain": domain_value(),
            "httpOnly": http_only_1,
            "name": cookie2_name,
            "path": "/",
            "sameSite": "none",
            "secure": True,
            "size": 6,
            "value": {"type": "string", "value": cookie2_value},
        },
        cookie_2,
    )
