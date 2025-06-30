import pytest, pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget
from .. import BATTERY_SERVICE_UUID, HEART_RATE_SERVICE_UUID, TEST_DEVICE_ADDRESS, create_gatt_connection, setup_granted_device, remote_array_to_list
from .... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture(autouse=True)
async def fixture(bidi_session, top_context, test_page, subscribe_events, wait_for_event):
    await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event, [HEART_RATE_SERVICE_UUID, BATTERY_SERVICE_UUID])
    await create_gatt_connection(bidi_session, top_context, subscribe_events, wait_for_event)
    yield
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])

async def test_simulate_service(bidi_session, top_context):
    async def get_services():
        return await bidi_session.script.call_function(
            function_declaration="""async ()=>{
                const devices = await navigator.bluetooth.getDevices();
                const device = devices[0];
                try {
                    const services = await device.gatt.getPrimaryServices();
                    return services.map(s => s.uuid);
                } catch (e) {
                    if (e.name === 'NotFoundError') {
                        return [];
                    }
                    throw e;
                }
            }
            """,
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        )

    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=HEART_RATE_SERVICE_UUID,
        type="add")
    services = await get_services()
    recursive_compare([HEART_RATE_SERVICE_UUID], remote_array_to_list(services))

    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=BATTERY_SERVICE_UUID,
        type="add")
    services = await get_services()
    recursive_compare(sorted([HEART_RATE_SERVICE_UUID, BATTERY_SERVICE_UUID]),
                      sorted(remote_array_to_list(services)))

    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=BATTERY_SERVICE_UUID,
        type="remove")
    services = await get_services()
    recursive_compare([HEART_RATE_SERVICE_UUID], remote_array_to_list(services))

    await bidi_session.bluetooth.simulate_service(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS,
        uuid=HEART_RATE_SERVICE_UUID,
        type="remove")
    services = await get_services()
    assert remote_array_to_list(services) == []
