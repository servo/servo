import pytest

from . import SOME_CLIENT_HINTS

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            contexts=value
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            contexts=[])


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            contexts=[value])


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            contexts=["_invalid_"],
        )


async def test_params_contexts_iframe(bidi_session, new_tab, get_test_page):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            contexts=[frames[0]["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            user_contexts=[value],
        )


async def test_params_contexts_and_user_contexts(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=SOME_CLIENT_HINTS,
            contexts=[top_context["context"]],
            user_contexts=["default"],
        )


async def test_params_client_hints_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=UNDEFINED,
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("dict", nullable=True))
async def test_params_client_hints_invalid_type(bidi_session, top_context,
        value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_client_hints_override(
            client_hints=value,
            contexts=[top_context["context"]],
        )
