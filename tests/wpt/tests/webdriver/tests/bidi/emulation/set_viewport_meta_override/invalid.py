import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            contexts=value,
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            contexts=[value],
        )


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            contexts=["_invalid_context_id_"],
        )


async def test_params_contexts_iframe(
    bidi_session,
    new_tab,
    url,
    create_iframe,
):
    iframe = await create_iframe(new_tab, url("/"))

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            contexts=[iframe["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            user_contexts=[value],
        )


async def test_params_user_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            user_contexts=["_invalid_user_context_id_"],
        )


async def test_params_contexts_and_user_contexts(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=True,
            contexts=[top_context["context"]],
            user_contexts=["default"],
        )


async def test_params_viewport_meta_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=UNDEFINED,
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("boolean", nullable=True))
async def test_params_viewport_meta_invalid_type(
    bidi_session,
    top_context,
    value,
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=value,
            contexts=[top_context["context"]],
        )


async def test_params_viewport_meta_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_viewport_meta_override(
            viewport_meta=False,
            contexts=[top_context["context"]],
        )

