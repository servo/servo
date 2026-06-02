import pytest
import webdriver.bidi.error as error
from .. import TEST_DEVICE_ADDRESS

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_gatt_connection_response(
            context=context,
            address=TEST_DEVICE_ADDRESS, code=0x0)


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_gatt_connection_response(
            context="UNKNOWN_CONTEXT",
            address=TEST_DEVICE_ADDRESS, code=0x0)


@pytest.mark.parametrize("address", [None, False, 42, {}, []])
async def test_address_invalid_type(bidi_session, top_context, address):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_gatt_connection_response(
            context=top_context,
            address=address, code=0x0)


@pytest.mark.parametrize("code", ["0", None, False, {}, []])
async def test_code_invalid_type(bidi_session, top_context, code):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_gatt_connection_response(
            context=top_context,
            address=TEST_DEVICE_ADDRESS, code=code)
