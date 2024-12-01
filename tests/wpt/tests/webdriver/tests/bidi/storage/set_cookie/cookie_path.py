import pytest
from .. import assert_cookie_is_set, assert_partition_key, create_cookie

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "path",
    [
        "/",
        "/some_path",
        "/some/nested/path",
        None
    ]
)
async def test_cookie_path(bidi_session, test_page, set_cookie, domain_value, path):
    set_cookie_result = await set_cookie(cookie=create_cookie(domain=domain_value(), path=path))

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    # `path` defaults to "/".
    expected_path = path if path is not None else "/"
    await assert_cookie_is_set(bidi_session, path=expected_path, domain=domain_value())
