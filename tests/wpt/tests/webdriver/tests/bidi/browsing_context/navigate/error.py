import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "url",
    [
        "thisprotocoldoesnotexist://",
        "https://doesnotexist.localhost/",
        "https://localhost:0",
    ],
    ids=[
        "protocol",
        "host",
        "port",
    ]
)
async def test_invalid_address(bidi_session, new_tab, url):
    await navigate_and_assert(bidi_session, new_tab, url, expected_error=True)
