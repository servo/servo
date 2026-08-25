import pytest
from .. import assert_cookie_is_set, create_cookie
from webdriver.bidi.modules.network import NetworkBase64Value, NetworkStringValue

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "str_value",
    [
        "simple_value",
        "special_symbols =!@#$%^&*()_+-{}[]|\\:\"'<>,.?/`~"
    ])
async def test_cookie_value_string(bidi_session, set_cookie, test_page, domain_value, str_value):
    value = NetworkStringValue(str_value)

    await set_cookie(cookie=create_cookie(domain=domain_value(), value=value))
    await assert_cookie_is_set(bidi_session, value=value, domain=domain_value())


@pytest.mark.parametrize(
    "base64_value, decoded_value",
    [
        ("Zm9v", "foo"),
        ("aGVsbG8gd29ybGQ=", "hello world"),
    ])
async def test_cookie_value_base64(bidi_session, set_cookie, test_page, domain_value, base64_value, decoded_value):
    value = NetworkBase64Value(base64_value)

    await set_cookie(cookie=create_cookie(domain=domain_value(), value=value))

    # Valid UTF-8 base64 values are returned as string type:
    # https://www.w3.org/TR/webdriver-bidi/#serialize-protocol-bytes
    await assert_cookie_is_set(
        bidi_session, value=NetworkStringValue(decoded_value), domain=domain_value()
    )
