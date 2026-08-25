# META: timeout=long

import pytest
import pytest_asyncio
import webdriver.bidi.error as error
from .. import MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID, HEART_RATE_SERVICE_UUID, BATTERY_SERVICE_UUID, TEST_DEVICE_ADDRESS, TEST_DEVICE_2_ADDRESS, setup_granted_device

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture(autouse=True)
async def fixture(bidi_session, top_context, test_page, subscribe_events, wait_for_event):
    await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event)
    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=HEART_RATE_SERVICE_UUID,
        type="add")
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=context,
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=None)


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context="UNKNOWN_CONTEXT",
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=None)


@pytest.mark.parametrize("address", [None, False, 42, {}, []])
async def test_address_invalid_type(bidi_session, top_context, address):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context,
            address=address,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=None)

async def test_address_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context,
            address=TEST_DEVICE_2_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=None)


@pytest.mark.parametrize("service_uuid", [None, False, 42, {}, []])
async def test_service_uuid_invalid_type(bidi_session, top_context, service_uuid):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context,
            address=TEST_DEVICE_ADDRESS,
            service_uuid=service_uuid,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=None)


async def test_service_uuid_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context,
            address=TEST_DEVICE_ADDRESS,
            service_uuid=BATTERY_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=None)


@pytest.mark.parametrize("characteristic_uuid", [None, False, 42, {}, []])
async def test_characteristic_uuid_invalid_type(bidi_session, top_context, characteristic_uuid):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context,
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=characteristic_uuid,
            type="write",
            code=0x0,
            data=None)


@pytest.mark.parametrize("type", [None, False, 42, {}, []])
async def test_type_invalid_type(bidi_session, top_context, type):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type=type,
            code=0x0,
            data=None)


async def test_type_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="unknown_value",
            code=0x0,
            data=None)

async def test_data_present_for_nonread(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="write",
            code=0x0,
            data=[1])

async def test_data_absent_for_read(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic_response(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            type="read",
            code=0x0,
            data=None)
