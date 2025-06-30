import pytest
import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget
from .. import BATTERY_SERVICE_UUID, MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID, DATE_TIME_CHARACTERISTIC_UUID, HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, create_gatt_connection, setup_granted_device, remote_array_to_list
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
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])

@pytest.mark.parametrize("property", [
    "broadcast", "read", "writeWithoutResponse", "write", "notify", "indicate",
    "authenticatedSignedWrites"
])
async def test_simulate_characteristic(bidi_session, top_context, property):
    async def get_characteristics(service_uuid):
        return await bidi_session.script.call_function(
            function_declaration=f'''async ()=>{{
                const devices = await navigator.bluetooth.getDevices();
                const device = devices[0];
                try {{
                    const service = await device.gatt.getPrimaryService('{service_uuid}');
                    const characteristics = await service.getCharacteristics();
                    return characteristics.map(c => c.uuid)
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

    async def get_characteristic_properties(service_uuid, characteristic_uuid):
        return await bidi_session.script.call_function(
            function_declaration=f'''async ()=>{{
                const toPropertyStrings = (properties) => {{
                    let propertyStrings = [];
                    if (properties.broadcast) {{
                        propertyStrings.push('broadcast');
                    }}
                    if (properties.read) {{
                        propertyStrings.push('read');
                    }}
                    if (properties.writeWithoutResponse) {{
                        propertyStrings.push('writeWithoutResponse');
                    }}
                    if (properties.write) {{
                        propertyStrings.push('write');
                    }}
                    if (properties.notify) {{
                        propertyStrings.push('notify');
                    }}
                    if (properties.indicate) {{
                        propertyStrings.push('indicate');
                    }}
                    if (properties.authenticatedSignedWrites) {{
                        propertyStrings.push('authenticatedSignedWrites');
                    }}
                    return propertyStrings;
                }};
                try {{
                    const devices = await navigator.bluetooth.getDevices();
                    const device = devices[0];
                    const service = await device.gatt.getPrimaryService('{service_uuid}');
                    const characteristic = await service.getCharacteristic('{characteristic_uuid}');
                    return toPropertyStrings(characteristic.properties);
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

    await bidi_session.bluetooth.simulate_characteristic(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        characteristic_properties={property: True},
        type="add")
    recursive_compare([DATE_TIME_CHARACTERISTIC_UUID],
                      remote_array_to_list(
                          await get_characteristics(HEART_RATE_SERVICE_UUID)))
    recursive_compare([property],
                      remote_array_to_list(
                          await get_characteristic_properties(
                              HEART_RATE_SERVICE_UUID, DATE_TIME_CHARACTERISTIC_UUID)))

    await bidi_session.bluetooth.simulate_characteristic(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        characteristic_properties={property: True},
        type="add")
    recursive_compare(sorted([DATE_TIME_CHARACTERISTIC_UUID, MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID]),
                      sorted(remote_array_to_list(
                          await get_characteristics(HEART_RATE_SERVICE_UUID))))
    recursive_compare([property],
                      remote_array_to_list(
                          await get_characteristic_properties(
                              HEART_RATE_SERVICE_UUID, MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID)))

    await bidi_session.bluetooth.simulate_characteristic(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID,
        characteristic_properties=None,
        type="remove")
    recursive_compare([DATE_TIME_CHARACTERISTIC_UUID],
                      remote_array_to_list(
                          await get_characteristics(HEART_RATE_SERVICE_UUID)))

    await bidi_session.bluetooth.simulate_characteristic(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        service_uuid=HEART_RATE_SERVICE_UUID,
        characteristic_uuid=DATE_TIME_CHARACTERISTIC_UUID,
        characteristic_properties=None,
        type="remove")
    assert remote_array_to_list(await get_characteristics(HEART_RATE_SERVICE_UUID)) == []
