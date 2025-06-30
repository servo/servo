# META: timeout=long

import pytest
import pytest_asyncio
import webdriver.bidi.error as error
from .. import HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, TEST_DEVICE_2_ADDRESS, setup_granted_device

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture(autouse=True)
async def fixture(bidi_session, top_context, test_page, subscribe_events, wait_for_event):
    await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event)
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=context,
            address=TEST_DEVICE_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type="add")


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_service(
            context="UNKNOWN_CONTEXT",
            address=TEST_DEVICE_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type="add")


@pytest.mark.parametrize("address", [None, False, 42, {}, []])
async def test_address_invalid_type(bidi_session, top_context, address):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=address,
            uuid=HEART_RATE_SERVICE_UUID,
            type="add")


async def test_address_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=TEST_DEVICE_2_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type="add")


@pytest.mark.parametrize("uuid", [None, False, 42, {}, []])
async def test_uuid_invalid_type(bidi_session, top_context, uuid):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            uuid=uuid,
            type="add")


async def test_uuid_add_already_exist(bidi_session, top_context):
    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=HEART_RATE_SERVICE_UUID,
        type="add")
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type="add")


async def test_uuid_remove_nonexist(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type="remove")


@pytest.mark.parametrize("type", [None, False, 42, {}, []])
async def test_type_invalid_type(bidi_session, top_context, type):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type=type)


async def test_type_unknown_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_service(
            context=top_context["context"],
            address=TEST_DEVICE_ADDRESS,
            uuid=HEART_RATE_SERVICE_UUID,
            type="unknown_value")
