import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_reference_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.create(
            type_hint="tab", reference_context=value
        )


async def test_params_reference_context_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.create(
            type_hint="tab", reference_context="foo"
        )


async def test_params_reference_context_non_top_level(
    bidi_session, test_page_same_origin_frame, top_context
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_page_same_origin_frame,
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree()

    assert len(all_contexts) == 1
    parent_info = all_contexts[0]
    assert len(parent_info["children"]) == 1
    child_info = parent_info["children"][0]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.create(
            type_hint="tab", reference_context=child_info["context"]
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.create(type_hint=value)


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_type_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.create(type_hint=value)


@pytest.mark.parametrize("value", ['', 42, {}, []])
async def test_params_background_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.create(type_hint="tab", background = value)
