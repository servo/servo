import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("state", [None, False, 42, {}, []])
async def test_state_invalid_type(bidi_session, top_context, state):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_adapter(
            context=top_context["context"], state=state)


@pytest.mark.parametrize("state", ["", "invalid"])
async def test_state_invalid_value(bidi_session, top_context, state):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_adapter(
            context=top_context["context"], state=state)


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_adapter(
            context=context, state="powered-on")


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_adapter(
            context="UNKNOWN_CONTEXT", state="powered-on")
