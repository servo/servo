import pytest
from urllib.parse import urlparse
from .. import assert_cookie_is_set, create_cookie, get_default_partition_key

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "protocol",
    [
        "http",
        "https",
    ]
)
async def test_page_protocols(bidi_session, set_cookie, get_test_page, protocol):
    url = get_test_page(protocol=protocol)
    domain = urlparse(url).hostname
    set_cookie_result = await set_cookie(cookie=create_cookie(domain=domain))

    assert set_cookie_result == {
        'partitionKey': (await get_default_partition_key(bidi_session)),
    }

    # Assert the cookie is actually set.
    await assert_cookie_is_set(bidi_session, domain=domain)
