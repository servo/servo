import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget, RealmTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("target", [None, False, "foo", 42, {}, []])
async def test_params_target_invalid_type(bidi_session, target):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.disown(
            handles=[],
            target=target)


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.disown(
            handles=[],
            target=ContextTarget(context))


@pytest.mark.parametrize("sandbox", [False, 42, {}, []])
async def test_params_sandbox_invalid_type(bidi_session, top_context, sandbox):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.disown(
            handles=[],
            target=ContextTarget(top_context["context"], sandbox))


async def test_params_context_unknown(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.disown(
            handles=[],
            target=ContextTarget("_UNKNOWN_"))


@pytest.mark.parametrize("realm", [None, False, 42, {}, []])
async def test_params_realm_invalid_type(bidi_session, realm):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.disown(
            handles=[],
            target=RealmTarget(realm))


async def test_params_realm_unknown(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.disown(
            handles=[],
            target=RealmTarget("_UNKNOWN_"))


@pytest.mark.parametrize("handles", [None, False, "foo", 42, {}])
async def test_params_handles_invalid_type(bidi_session, top_context, handles):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.disown(
            handles=handles,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("handle", [None, False, 42, {}, []])
async def test_params_handles_invalid_handle_type(bidi_session, top_context, handle):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.disown(
            handles=[handle],
            target=ContextTarget(top_context["context"]))
