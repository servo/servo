import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=context, address="09:09:09:09:09:09", name="Some Device",
            manufacturer_data=[{"key": 17, "data": "AP8BAX8="}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])


async def test_context_unknown_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context="UNKNOWN_CONTEXT", address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=[{"key": 17, "data": "AP8BAX8="}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])


@pytest.mark.parametrize("address", [None, False, 42, {}, []])
async def test_address_invalid_type(bidi_session, top_context, address):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address=address, name="Some Device",
            manufacturer_data=[{"key": 17, "data": "AP8BAX8="}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])


@pytest.mark.parametrize("name", [None, False, 42, {}, []])
async def test_name_invalid_type(bidi_session, top_context, name):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name=name,
            manufacturer_data=[{"key": 17, "data": "AP8BAX8="}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])


@pytest.mark.parametrize("manufacturer_data", [None, False, 42, {}, ""])
async def test_manufacturer_data_invalid_type(bidi_session, top_context,
                                              manufacturer_data):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=manufacturer_data,
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])

@pytest.mark.parametrize("key", [None, False, [], {}, ""])
async def test_manufacturer_data_invalid_key_type(bidi_session, top_context, key):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=[{"key": key, "data": "AP8BAX8="}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])

async def test_manufacturer_data_invalid_key_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=[{"key": -1, "data": "AP8BAX8="}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])


@pytest.mark.parametrize("data", [None, False, 42, {}, []])
async def test_manufacturer_data_invalid_data_type(bidi_session, top_context,
                                                   data):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=[{"key": 17, "data": data}],
            known_service_uuids=["12345678-1234-5678-9abc-def123456789"])


@pytest.mark.parametrize("known_service_uuids", [None, False, 42, {}])
async def test_known_service_uuids_invalid_type(bidi_session, top_context,
                                                known_service_uuids):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=[{"key": 17, "data": "AP8BAX8="}],
            known_service_uuids=known_service_uuids)


@pytest.mark.parametrize("uuid", [None, False, 42, {}, []])
async def test_known_service_uuids_invalid_uuid_type(bidi_session, top_context,
                                                     uuid):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
            context=top_context["context"], address="09:09:09:09:09:09",
            name="Some Device",
            manufacturer_data=[{"key": 17, "data": "AP8BAX8="}],
            known_service_uuids=[uuid])
