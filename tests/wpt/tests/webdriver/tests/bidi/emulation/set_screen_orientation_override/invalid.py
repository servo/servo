import pytest

import webdriver.bidi.error as error
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=value,
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            })


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[],
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            })


@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_contexts_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[value],
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            })


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=["_invalid_"],
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            })


async def test_params_contexts_iframe(bidi_session, new_tab, get_test_page):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    frames = contexts[0]["children"]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[frames[0]["context"]],
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
        })


@pytest.mark.parametrize("value", [False, "foo", 42, []])
async def test_params_screen_orientation_invalid_type(bidi_session, top_context,
        value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[top_context["context"]],
            screen_orientation=value
        )


async def test_params_screen_orientation_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[top_context["context"]],
            screen_orientation=UNDEFINED
        )


@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_screen_orientation_natural_invalid_type(bidi_session,
        top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[top_context["context"]],
            screen_orientation={
                "natural": value,
                "type": "portrait-primary"
            })


async def test_params_screen_orientation_natural_invalid_value(bidi_session,
        top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[top_context["context"]],
            screen_orientation={
                "natural": "invalid natural screen orientation",
                "type": "portrait-primary"
            })


@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_screen_orientation_type_invalid_type(bidi_session,
        top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[top_context["context"]],
            screen_orientation={
                "natural": "portrait",
                "type": value
            })


async def test_params_screen_orientation_type_invalid_value(bidi_session,
        top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            contexts=[top_context["context"]],
            screen_orientation={
                "natural": "portrait",
                "type": "invalid type screen orientation"
            })


async def test_params_contexts_and_user_contexts(bidi_session,
        top_context, create_user_context):
    user_context = await create_user_context()
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            },
            contexts=[top_context["context"]],
            user_contexts=[user_context])


@pytest.mark.parametrize("value", [None, False, "foo", 42, {}])
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            },
            user_contexts=value)


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            },
            user_contexts=[])


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_screen_orientation_override(
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            },
            user_contexts=[value])


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_screen_orientation_override(
            screen_orientation={
                "natural": "portrait",
                "type": "portrait-primary"
            },
            user_contexts=[value])
