import pytest
import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget
from .. import CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID, CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID, MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID, DATE_TIME_CHARACTERISTIC_UUID, HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, create_gatt_connection, setup_granted_device, remote_array_to_list
from .... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture(autouse=True)
async def fixture(bidi_session, top_context, test_page, subscribe_events, wait_for_event):
    await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event, [HEART_RATE_SERVICE_UUID])
    await create_gatt_connection(bidi_session, top_context, subscribe_events, wait_for_event)
    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=HEART_RATE_SERVICE_UUID,
        type="add")
    await bidi_session.bluetooth.simulate_characteristic(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        characteristic_properties={"read": True},
        type="add")
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])


async def test_simulate_descriptor(bidi_session, top_context):
    async def get_descriptors(service_uuid, characteristic_uuid):
        return await bidi_session.script.call_function(
            function_declaration=f'''async ()=>{{
                const devices = await navigator.bluetooth.getDevices();
                const device = devices[0];
                try {{
                    const service = await device.gatt.getPrimaryService('{service_uuid}');
                    const characteristic = await service.getCharacteristic('{characteristic_uuid}');
                    const descriptors = await characteristic.getDescriptors();
                    return descriptors.map(d => d.uuid)
                }} catch (e) {{
                    if (e.name === 'NotFoundError') {{
                        return [];
                    }}
                    throw e;
                }}
            }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        )

    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        descriptor_uuid=CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        type="add")
    recursive_compare([CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID],
                      remote_array_to_list(
                          await get_descriptors(HEART_RATE_SERVICE_UUID,
                                                MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID)))

    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        descriptor_uuid=CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID,
        type="add")
    recursive_compare(sorted([CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
                              CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID]),
                      sorted(remote_array_to_list(
                          await get_descriptors(HEART_RATE_SERVICE_UUID,
                                                MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID))))

    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        descriptor_uuid=CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID,
        type="remove")
    recursive_compare([CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID],
                      remote_array_to_list(
                          await get_descriptors(HEART_RATE_SERVICE_UUID,
                                                MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID)))

    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        descriptor_uuid=CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        type="remove")
    assert remote_array_to_list(await get_descriptors(
        HEART_RATE_SERVICE_UUID, MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID)) == []

