import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            contexts=value
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            contexts=[])


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            contexts=[value])


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            contexts=["_invalid_"],
        )


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            user_contexts=[value],
        )


async def test_params_contexts_and_user_contexts(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=None,
            contexts=[top_context["context"]],
            user_contexts=["default"],
        )


async def test_params_touch_override_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=UNDEFINED,
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("number", nullable=True))
async def test_params_touch_override_invalid_type(bidi_session, top_context,
                                                  value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=value,
            contexts=[top_context["context"]],
        )


async def test_params_touch_override_invalid_value(bidi_session,
                                                   top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_touch_override(
            max_touch_points=-1,
            contexts=[top_context["context"]],
        )
