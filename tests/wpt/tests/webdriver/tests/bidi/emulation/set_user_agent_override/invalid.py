import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio

SOME_USER_AGENT = "SOME_USER_AGENT"


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            contexts=value
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            contexts=[])


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            contexts=[value])


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
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
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            contexts=[frames[0]["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            user_contexts=[value],
        )


async def test_params_contexts_and_user_contexts(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=SOME_USER_AGENT,
            contexts=[top_context["context"]],
            user_contexts=["default"],
        )


async def test_params_user_agent_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=UNDEFINED,
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string", nullable=True))
async def test_params_user_agent_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_user_agent_override(
            user_agent=value,
            contexts=[top_context["context"]],
        )
