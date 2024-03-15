import pytest
from webdriver.bidi.modules.network import NetworkBase64Value, NetworkStringValue
from webdriver.bidi.modules.storage import CookieFilter

from . import assert_cookies_are_not_present
from .. import (
    assert_cookie_is_set,
    create_cookie,
    format_expiry_string,
    get_default_partition_key,
    generate_expiry_date,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "filter",
    [
        ({"size": 6}),
        ({"value": NetworkStringValue("bar")}),
        ({"value": NetworkBase64Value("YmFy")}),
    ],
)
async def test_filter(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    add_cookie,
    filter,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )
    # This value is going to be used to match specified filter options:
    # 1. size == 6, both matching cookies have name length 3, the value has length 3;
    # 2. value == "bar";
    # 3. value == base64 value "YmFy", it will be decoded to a string "bar".
    cookie_value_matching_filter = "bar"

    cookie1_name = "baz"
    await add_cookie(new_tab["context"], cookie1_name, cookie_value_matching_filter)

    cookie2_name = "foo"
    await add_cookie(new_tab["context"], cookie2_name, cookie_value_matching_filter)

    cookie3_name = "foo_3"
    cookie3_value = "not_bar"
    await add_cookie(new_tab["context"], cookie3_name, cookie3_value)

    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present.
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        path="/webdriver/tests/support",
        secure=False,
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
    cookie3_domain = domain_value(domain="alt")
    await add_cookie(new_tab["context"], cookie3_name, cookie3_value)

    filter = CookieFilter(domain=domain_value())
    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present.
    await assert_cookie_is_set(
        bidi_session,
        domain=cookie3_domain,
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        path="/webdriver/tests/support",
        secure=False,
    )


@pytest.mark.parametrize(
    "expiry_diff_to_delete, expiry_diff_to_remain",
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
    expiry_diff_to_delete,
    expiry_diff_to_remain,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    expiry_date_to_delete = generate_expiry_date(expiry_diff_to_delete)
    expiry_to_delete = int(expiry_date_to_delete.timestamp())
    date_string_to_delete = format_expiry_string(expiry_date_to_delete)

    cookie1_name = "bar"
    cookie1_value = "foo"
    await add_cookie(
        context=new_tab["context"],
        name=cookie1_name,
        value=cookie1_value,
        expiry=date_string_to_delete,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        context=new_tab["context"],
        name=cookie2_name,
        value=cookie2_value,
        expiry=date_string_to_delete,
    )

    cookie3_name = "foo_3"
    cookie3_value = "bar_3"
    if expiry_diff_to_remain is None:
        date_string_to_remain = None
    else:
        expiry_date_to_remain = generate_expiry_date(expiry_diff_to_remain)
        date_string_to_remain = format_expiry_string(expiry_date_to_remain)

    await add_cookie(
        new_tab["context"], cookie3_name, cookie3_value, expiry=date_string_to_remain
    )

    filter = CookieFilter(expiry=expiry_to_delete)
    result = await bidi_session.storage.delete_cookies(
        filter=CookieFilter(expiry=expiry_to_delete),
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present.
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        path="/webdriver/tests/support",
        secure=False,
    )


async def test_filter_name(bidi_session, new_tab, test_page, add_cookie, domain_value):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page, wait="complete"
    )

    name_to_delete = "foo"

    cookie1_value = "bar"
    cookie1_path = "/"
    await add_cookie(
        new_tab["context"], name_to_delete, cookie1_value, path=cookie1_path
    )

    cookie2_value = "baz"
    cookie2_path = "/webdriver/"
    await add_cookie(
        new_tab["context"], name_to_delete, cookie2_value, path=cookie2_path
    )

    name_to_remain = "foo_2"
    cookie3_value = "bar_2"
    cookie3_path = "/"
    await add_cookie(new_tab["context"], name_to_remain, cookie3_value, path=cookie3_path)

    filter = CookieFilter(name=name_to_delete)
    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present.
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=name_to_remain,
        value={"type": "string", "value": cookie3_value},
        path=cookie3_path,
        secure=False,
    )


@pytest.mark.parametrize(
    "same_site_to_delete, same_site_to_remain",
    [
        ("none", "strict"),
        ("lax", "none"),
        ("strict", "none"),
        ("lax", "strict"),
        ("strict", "lax"),
    ],
)
async def test_filter_same_site(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    same_site_to_delete,
    same_site_to_remain,
    add_cookie,
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
        same_site=same_site_to_delete,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        new_tab["context"],
        cookie2_name,
        cookie2_value,
        same_site=same_site_to_delete,
    )

    cookie3_name = "foo_3"
    cookie3_value = "bar_3"
    await add_cookie(
        new_tab["context"], cookie3_name, cookie3_value, same_site=same_site_to_remain
    )

    filter = CookieFilter(same_site=same_site_to_delete)
    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        path="/webdriver/tests/support",
        same_site=same_site_to_remain,
        secure=False,
    )


@pytest.mark.parametrize(
    "secure_to_delete, secure_to_remain",
    [(True, False), (False, True)],
)
async def test_filter_secure(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    add_cookie,
    secure_to_delete,
    secure_to_remain,
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
        secure=secure_to_delete,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        new_tab["context"],
        cookie2_name,
        cookie2_value,
        secure=secure_to_delete,
    )

    cookie3_name = "foo_3"
    cookie3_value = "bar_3"
    await add_cookie(
        new_tab["context"], cookie3_name, cookie3_value, secure=secure_to_remain
    )

    filter = CookieFilter(secure=secure_to_delete)
    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present.
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        path="/webdriver/tests/support",
        secure=secure_to_remain,
    )


@pytest.mark.parametrize(
    "path_to_delete, path_to_remain",
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
    path_to_delete,
    path_to_remain,
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
        path=path_to_delete,
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await add_cookie(
        new_tab["context"],
        cookie2_name,
        cookie2_value,
        path=path_to_delete,
    )

    cookie3_name = "foo_3"
    cookie3_value = "bar_3"
    await add_cookie(
        new_tab["context"], cookie3_name, cookie3_value, path=path_to_remain
    )

    filter = CookieFilter(path=path_to_delete)
    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present.
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        path="/webdriver/tests/support" if path_to_remain is None else path_to_remain,
        secure=False,
    )


@pytest.mark.parametrize(
    "http_only_to_delete, http_only_to_remain",
    [(True, False), (False, True)],
)
async def test_filter_http_only(
    bidi_session,
    new_tab,
    test_page,
    domain_value,
    set_cookie,
    http_only_to_delete,
    http_only_to_remain,
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
            http_only=http_only_to_delete,
        )
    )

    cookie2_name = "foo"
    cookie2_value = "bar"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            name=cookie2_name,
            value=NetworkStringValue(cookie2_value),
            http_only=http_only_to_delete,
        )
    )

    cookie3_name = "foo_2"
    cookie3_value = "bar_2"
    await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            name=cookie3_name,
            value=NetworkStringValue(cookie3_value),
            http_only=http_only_to_remain,
        )
    )

    filter = CookieFilter(http_only=http_only_to_delete)
    result = await bidi_session.storage.delete_cookies(
        filter=filter,
    )
    assert result == {
        "partitionKey": (await get_default_partition_key(bidi_session))
    }

    # Make sure that deleted cookies are not present.
    await assert_cookies_are_not_present(bidi_session, filter)

    # Make sure that non-deleted cookies are present
    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        name=cookie3_name,
        value={"type": "string", "value": cookie3_value},
        http_only=http_only_to_remain,
    )
