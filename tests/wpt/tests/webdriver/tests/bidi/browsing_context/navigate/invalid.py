import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, inline, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.navigate(
            context=value, url=inline("<p>foo")
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_context_invalid_value(bidi_session, inline, value):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.navigate(
            context=value, url=inline("<p>foo")
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_url_invalid_type(bidi_session, new_tab, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=value
        )


@pytest.mark.parametrize(
    "url",
    [
        "1foo://",
        "http://foo bar",
        "http://localhost:-1",
        "http://:80",
        "/foo",
        "http://#foo",
    ],
    ids=[
        "protocol",
        "host",
        "port",
        "port without host",
        "path absolute",
        "hash without host",
    ],
)
async def test_params_url_invalid_value(bidi_session, new_tab, url):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=url
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_wait_invalid_type(bidi_session, inline, new_tab, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=inline("<p>bar"), wait=value
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_wait_invalid_value(bidi_session, inline, new_tab, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=inline("<p>bar"), wait=value
        )
