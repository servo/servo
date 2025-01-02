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


@pytest.mark.parametrize("protocol", ["http", "https"])
@pytest.mark.parametrize("value", [":invalid", "#invalid"])
async def test_params_url_invalid_value(bidi_session, new_tab, protocol, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=f"{protocol}://{value}"
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
