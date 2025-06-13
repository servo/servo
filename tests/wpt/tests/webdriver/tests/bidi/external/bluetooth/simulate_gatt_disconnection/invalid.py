import pytest
import webdriver.bidi.error as error
from .. import TEST_DEVICE_ADDRESS

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_gatt_disconnection(
            context=context,
            address=TEST_DEVICE_ADDRESS)


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_gatt_disconnection(
            context="UNKNOWN_CONTEXT",
            address=TEST_DEVICE_ADDRESS)


@pytest.mark.parametrize("address", [None, False, 42, {}, []])
async def test_address_invalid_type(bidi_session, top_context, address):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_gatt_disconnection(
            context=top_context,
            address=address)
