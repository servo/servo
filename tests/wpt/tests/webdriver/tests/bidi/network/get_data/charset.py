import pytest

from .. import PAGE_EMPTY_IMAGE, PAGE_EMPTY_TEXT, PAGE_OTHER_TEXT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "charset", [None, "utf-8", "iso-8859-15", "fakeCharset"]
)
async def test_request_charset(
    bidi_session,
    url,
    setup_collected_response,
    charset,
):
    expected_text = "Ü (lowercase ü)"

    if charset is None:
        test_url = url(
            f"/webdriver/tests/support/http_handlers/charset.py?content={expected_text}"
        )
    else:
        test_url = url(
            f"/webdriver/tests/support/http_handlers/charset.py?content={expected_text}&charset={charset}"
        )

    [request, _] = await setup_collected_response(fetch_url=test_url)
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "string"
    # Regardless of the charset provided in the response, the text should always
    # be decoded as utf-8.
    assert data["value"] == expected_text
