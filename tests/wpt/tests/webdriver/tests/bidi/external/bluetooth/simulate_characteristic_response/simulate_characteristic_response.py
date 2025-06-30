# META: timeout=long

import pytest, asyncio
import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget
from .. import BATTERY_SERVICE_UUID, CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID, CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID, BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT, MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID, DATE_TIME_CHARACTERISTIC_UUID, HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, create_gatt_connection, setup_granted_device, remote_array_to_list
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
        characteristic_properties={"read": True,
                                   "write": True,
                                   "notify": True},
        type="add")
    await subscribe_events(
            events=[BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT])
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])


@pytest.mark.parametrize(
    'write_type', [('writeValueWithoutResponse', 'write-without-response'),
                   ('writeValueWithResponse', 'write-with-response')])
async def test_simulate_characteristic_write_event(bidi_session, top_context, subscribe_events, wait_for_event, write_type):
    expected_data = [1]
    asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f'''
                async () => {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{HEART_RATE_SERVICE_UUID}');
                    const characteristic = await service.getCharacteristic('{DATE_TIME_CHARACTERISTIC_UUID}');
                    await characteristic.{write_type[0]}(new Uint8Array({expected_data}));
                }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    characteristic_write_event = await wait_for_event(
        BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": TEST_DEVICE_ADDRESS,
        "serviceUuid": HEART_RATE_SERVICE_UUID,
        'characteristicUuid': DATE_TIME_CHARACTERISTIC_UUID,
        'type': write_type[1],
        'data': expected_data,
    }, characteristic_write_event)

    await bidi_session.bluetooth.simulate_characteristic_response(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        type='write',
        code=0x0,
        data=None)


async def test_simulate_characteristic_read_event(bidi_session, top_context, subscribe_events, wait_for_event):
    characteristic_read_future = asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f'''
                async () => {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{HEART_RATE_SERVICE_UUID}');
                    const characteristic = await service.getCharacteristic('{DATE_TIME_CHARACTERISTIC_UUID}');
                    const value = await characteristic.readValue();
                    return Array.from(new Uint8Array(value.buffer));
                }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    characteristic_read_event = await wait_for_event(
        BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": TEST_DEVICE_ADDRESS,
        "serviceUuid": HEART_RATE_SERVICE_UUID,
        'characteristicUuid': DATE_TIME_CHARACTERISTIC_UUID,
        'type': "read"
    }, characteristic_read_event)

    expected_data = [1, 2]
    await bidi_session.bluetooth.simulate_characteristic_response(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        type='read',
        code=0x0,
        data=expected_data)

    characteristic_read = await characteristic_read_future
    recursive_compare(expected_data, remote_array_to_list(characteristic_read))


async def test_simulate_characteristic_notification_event(bidi_session, top_context, subscribe_events, wait_for_event):
    # The following two descriptors are needed for testing start and stop
    # notification subscription.
    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        descriptor_uuid=CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID,
        type="add")
    await bidi_session.bluetooth.simulate_descriptor(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        descriptor_uuid=CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID,
        type="add")

    # Test start notification.
    asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f'''
                async () => {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{HEART_RATE_SERVICE_UUID}');
                    const characteristic = await service.getCharacteristic('{DATE_TIME_CHARACTERISTIC_UUID}');
                    await characteristic.startNotifications();
                }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    characteristic_start_notification_event = await wait_for_event(
        BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": TEST_DEVICE_ADDRESS,
        "serviceUuid": HEART_RATE_SERVICE_UUID,
        'characteristicUuid': DATE_TIME_CHARACTERISTIC_UUID,
        'type': "subscribe-to-notifications"
    }, characteristic_start_notification_event)

    await bidi_session.bluetooth.simulate_characteristic_response(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        type='subscribe-to-notifications',
        code=0x0,
        data=None)

    # Test stop notification.
    asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f'''
                async () => {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{HEART_RATE_SERVICE_UUID}');
                    const characteristic = await service.getCharacteristic('{DATE_TIME_CHARACTERISTIC_UUID}');
                    await characteristic.stopNotifications();
                }}
            ''',
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    characteristic_stop_notification_event = await wait_for_event(
        BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": TEST_DEVICE_ADDRESS,
        "serviceUuid": HEART_RATE_SERVICE_UUID,
        'characteristicUuid': DATE_TIME_CHARACTERISTIC_UUID,
        'type': "unsubscribe-from-notifications"
    }, characteristic_stop_notification_event)

    await bidi_session.bluetooth.simulate_characteristic_response(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        type='unsubscribe-from-notifications',
        code=0x0,
        data=None)