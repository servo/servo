import pytest
from .. import assert_cookie_is_set, create_cookie

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "same_site",
    [
        "strict",
        "lax",
        "none",
        None
    ]
)
async def test_cookie_secure(bidi_session, test_page, domain_value, same_site):
    set_cookie_result = await bidi_session.storage.set_cookie(
        cookie=create_cookie(domain=domain_value(), same_site=same_site))

    assert set_cookie_result == {
        'partitionKey': {},
    }

    # `same_site` defaults to "none".
    expected_same_site = same_site if same_site is not None else 'none'
    await assert_cookie_is_set(bidi_session, domain=domain_value(), same_site=expected_same_site)
