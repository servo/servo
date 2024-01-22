import pytest
from .. import assert_cookie_is_set, create_cookie
from webdriver.bidi.modules.network import NetworkStringValue

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "str_value",
    [
        "simple_value",
        "special_symbols =!@#$%^&*()_+-{}[]|\\:\"'<>,.?/`~"
    ])
async def test_cookie_value_string(bidi_session, test_page, domain_value, str_value):
    value = NetworkStringValue(str_value)

    await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), value=value))
    await assert_cookie_is_set(bidi_session, value=value, domain=domain_value())

# TODO: test `test_cookie_value_base64`.
