import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio

SOME_SCREEN_AREA = {"width": 100, "height": 100}


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA, contexts=value
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA, contexts=[]
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA, contexts=[value]
        )


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            contexts=["_invalid_"],
        )


async def test_params_contexts_iframe(bidi_session, new_tab, get_test_page):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    frames = contexts[0]["children"]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            contexts=[frames[0]["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("dict", nullable=True))
async def test_params_screen_area_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=value,
            contexts=[top_context["context"]],
        )


async def test_params_screen_area_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=UNDEFINED,
            contexts=[top_context["context"]],
        )


async def test_params_screen_area_with_empty_object(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area={},
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("number", nullable=True))
async def test_params_screen_area_height_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area={"width": 100, "height": value},
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("number", nullable=True))
async def test_params_screen_area_width_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area={"width": value, "height": 100},
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            user_contexts=[value],
        )


async def test_params_contexts_and_user_contexts(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_settings_override(
            screen_area=SOME_SCREEN_AREA,
            contexts=[top_context["context"]],
            user_contexts=["default"],
        )
