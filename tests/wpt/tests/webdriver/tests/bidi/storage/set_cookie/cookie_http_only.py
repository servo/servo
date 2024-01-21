import pytest
from .. import assert_cookie_is_set, create_cookie

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "http_only",
    [
        True,
        False,
        None
    ])
async def test_cookie_http_only(bidi_session, test_page, domain_value, http_only):
    set_cookie_result = await bidi_session.storage.set_cookie(
        cookie=create_cookie(domain=domain_value(), http_only=http_only))

    assert set_cookie_result == {
        'partitionKey': {},
    }

    # `httpOnly` defaults to `false`.
    expected_http_only = http_only if http_only is not None else False

    await assert_cookie_is_set(
        bidi_session,
        domain=domain_value(),
        http_only=expected_http_only,
    )
