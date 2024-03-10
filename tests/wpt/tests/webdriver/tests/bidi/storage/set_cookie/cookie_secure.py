import pytest
from .. import assert_cookie_is_set, create_cookie, get_default_partition_key

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "secure",
    [
        True,
        False,
        None
    ]
)
async def test_cookie_secure(bidi_session, set_cookie, test_page, domain_value, secure):
    set_cookie_result = await set_cookie(
        cookie=create_cookie(domain=domain_value(), secure=secure))

    assert set_cookie_result == {
        'partitionKey': (await get_default_partition_key(bidi_session)),
    }

    # `secure` defaults to `false`.
    expected_secure = secure if secure is not None else False
    await assert_cookie_is_set(bidi_session, domain=domain_value(), secure=expected_secure)
