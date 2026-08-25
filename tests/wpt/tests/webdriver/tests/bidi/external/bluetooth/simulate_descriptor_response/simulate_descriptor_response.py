# META: timeout=long

import pytest, asyncio
import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget
from .. import BATTERY_SERVICE_UUID, CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID, BLUETOOTH_DESCRIPTOR_EVENT_GENERATED_EVENT, DATE_TIME_CHARACTERISTIC_UUID, HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, create_gatt_connection, setup_granted_device, remote_array_to_list
from .... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture(autouse=True)
async def fixture(bidi_session, top_context, test_page, subscribe_events, wait_for_event):
    await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event, [HEART_RATE_SERVICE_UUID, BATTERY_SERVICE_UUID])
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
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        characteristic_properties={"write": True},
        type="add")
    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        descriptor_uuid=CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        type="add")
    await subscribe_events(
            events=[BLUETOOTH_DESCRIPTOR_EVENT_GENERATED_EVENT])
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])


async def test_simulate_descriptor_write_event(bidi_session, top_context, subscribe_events, wait_for_event):
    expected_data = [1]
    asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f'''
                async () => {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{HEART_RATE_SERVICE_UUID}');
                    const characteristic = await service.getCharacteristic('{DATE_TIME_CHARACTERISTIC_UUID}');
                    const descriptor = await characteristic.getDescriptor('{CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID}');
                    await descriptor.writeValue(new Uint8Array({expected_data}));
                }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    descriptor_write_event = await wait_for_event(
        BLUETOOTH_DESCRIPTOR_EVENT_GENERATED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": TEST_DEVICE_ADDRESS,
        "serviceUuid": HEART_RATE_SERVICE_UUID,
        'characteristicUuid': DATE_TIME_CHARACTERISTIC_UUID,
        'descriptorUuid': CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        'type': 'write',
        'data': expected_data,
    }, descriptor_write_event)

    await bidi_session.bluetooth.simulate_descriptor_response(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        descriptor_uuid=CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        type='write',
        code=0x0,
        data=None)


async def test_simulate_characteristic_read_event(bidi_session, top_context, subscribe_events, wait_for_event):
    descriptor_read_future = asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f'''
                async () => {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{HEART_RATE_SERVICE_UUID}');
                    const characteristic = await service.getCharacteristic('{DATE_TIME_CHARACTERISTIC_UUID}');
                    const descriptor = await characteristic.getDescriptor('{CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID}');
                    const value = await descriptor.readValue();
                    return Array.from(new Uint8Array(value.buffer));
                }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    descriptor_read_event = await wait_for_event(
        BLUETOOTH_DESCRIPTOR_EVENT_GENERATED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": TEST_DEVICE_ADDRESS,
        "serviceUuid": HEART_RATE_SERVICE_UUID,
        'characteristicUuid': DATE_TIME_CHARACTERISTIC_UUID,
        'descriptorUuid': CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        'type': 'read',
    }, descriptor_read_event)

    expected_data = [1, 2]
    await bidi_session.bluetooth.simulate_descriptor_response(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        descriptor_uuid=CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        type='read',
        code=0x0,
        data=expected_data)

    descriptor_read = await descriptor_read_future
    recursive_compare(expected_data, remote_array_to_list(descriptor_read))
