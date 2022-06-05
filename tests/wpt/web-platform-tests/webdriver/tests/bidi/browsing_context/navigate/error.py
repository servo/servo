import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "url",
    [
        "thisprotocoldoesnotexist://",
        "http://doesnotexist.localhost/",
        "http://localhost:0",
    ],
    ids=[
        "protocol",
        "host",
        "port",
    ]
)
async def test_invalid_address(bidi_session, new_tab, url):
    await navigate_and_assert(bidi_session, new_tab, url, expected_error=True)


async def test_invalid_content_encoding(bidi_session, new_tab, inline):
    await navigate_and_assert(
        bidi_session,
        new_tab,
        f"{inline('<div>foo')}&pipe=header(Content-Encoding,gzip)",
        expected_error=True
    )
