import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio

@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.activate(
            context=value
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_context_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.activate(
            context=value
        )


@pytest.mark.asyncio
async def test_params_context_iframe(bidi_session, new_tab, get_test_page):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete")

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1
    frame_context = frames[0]["context"]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.activate(context=frame_context)
