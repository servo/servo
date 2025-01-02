import pytest
from .. import assert_cookie_is_set, assert_partition_key, create_cookie

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

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    # `secure` defaults to `false`.
    expected_secure = secure if secure is not None else False
    await assert_cookie_is_set(bidi_session, domain=domain_value(), secure=expected_secure)
