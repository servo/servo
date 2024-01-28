import pytest
from .. import assert_cookie_is_set, create_cookie

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "name",
    [
        "",
        "cookie name with special symbols !@#$%&*()_+-{}[]|\\:\"'<>,.?/`~",
        "123cookie",
    ])
async def test_cookie_name(bidi_session, set_cookie, test_page, domain_value, name):
    await set_cookie(cookie=create_cookie(domain=domain_value(), name=name))
    await assert_cookie_is_set(bidi_session, name=name, domain=domain_value())
