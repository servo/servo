import pytest
import webdriver.bidi.error as error


pytestmark = pytest.mark.asyncio


MAX_INT = 9007199254740991
MIN_INT = -MAX_INT


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.traverse_history(context=value, delta=1)


async def test_params_context_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.traverse_history(context="foo", delta=1)


@pytest.mark.parametrize(
    "value", [None, False, "foo", 1.5, MIN_INT - 1, MAX_INT + 1, {}, []]
)
async def test_params_delta_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.traverse_history(
            context=top_context["context"], delta=value
        )


@pytest.mark.parametrize("value", [-2, 1])
async def test_delta_invalid_value(bidi_session, current_url, new_tab, inline, value):
    page = inline("<div>page 1</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page, wait="complete"
    )
    assert await current_url(new_tab["context"]) == page

    with pytest.raises(error.NoSuchHistoryEntryException):
        await bidi_session.browsing_context.traverse_history(
            context=new_tab["context"], delta=value
        )


async def test_iframe(bidi_session, current_url, wait_for_url, new_tab, inline):
    iframe_url_1 = inline("page 1")
    page_url = inline(f"<iframe src='{iframe_url_1}'></iframe>")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.traverse_history(
            context=iframe_context["context"], delta=-1
        )
