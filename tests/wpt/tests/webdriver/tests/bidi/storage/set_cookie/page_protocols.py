import pytest
from urllib.parse import urlparse
from .. import assert_cookie_is_set, assert_partition_key, create_cookie

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

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    # Assert the cookie is actually set.
    await assert_cookie_is_set(bidi_session, domain=domain)
