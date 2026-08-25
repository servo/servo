# META: timeout=long

import pytest
import pytest_asyncio
import webdriver.bidi.error as error
from .. import MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID, BATTERY_SERVICE_UUID, HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, TEST_DEVICE_2_ADDRESS, setup_granted_device

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
        await bidi_session.bluetooth.simulate_characteristic(
            context=context,
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_characteristic(
            context="UNKNOWN_CONTEXT",
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")


@pytest.mark.parametrize("address", [None, False, 42, {}, []])
async def test_address_invalid_type(bidi_session, top_context, address):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=address,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")

async def test_address_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_2_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")


@pytest.mark.parametrize("service_uuid", [None, False, 42, {}, []])
async def test_service_uuid_invalid_type(bidi_session, top_context, service_uuid):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=service_uuid,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")


async def test_service_uuid_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=BATTERY_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")


@pytest.mark.parametrize("characteristic_uuid", [None, False, 42, {}, []])
async def test_characteristic_uuid_invalid_type(bidi_session, top_context, characteristic_uuid):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=characteristic_uuid,
            characteristic_properties={"read": True},
            type="add")


async def test_characteristic_uuid_add_already_exist(bidi_session, top_context):
    await bidi_session.bluetooth.simulate_characteristic(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        characteristic_properties={"read": True},
        type="add")
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="add")


async def test_characteristic_uuid_remove_nonexist(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties=None,
            type="remove")


@pytest.mark.parametrize("characteristic_properties", [None, False, 42, "properties", []])
async def test_characteristic_properties_invalid_type(bidi_session, top_context, characteristic_properties):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties=characteristic_properties,
            type="add")


@pytest.mark.parametrize("propertie_value", [None, 42, "True", []])
async def test_characteristic_properties_invalid_property_value(bidi_session, top_context, propertie_value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": propertie_value},
            type="add")


async def test_characteristic_properties_absent_for_add(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties=None,
            type="add")


async def test_characteristic_properties_present_for_remove(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties={"read": True},
            type="remove")


@pytest.mark.parametrize("type", [None, False, 42, {}, []])
async def test_type_invalid_type(bidi_session, top_context, type):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties=None,
            type=type)


async def test_type_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_characteristic(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            service_uuid=HEART_RATE_SERVICE_UUID,
            characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
            characteristic_properties=None,
            type="unknown_value")
