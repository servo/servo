import pytest, asyncio

from webdriver.bidi.modules.script import ContextTarget
from .. import BLUETOOTH_GATT_CONNECTION_ATTEMPTED_EVENT, setup_granted_device
from .... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize('code', [0x0, 0x1, 0x2])
async def test_simulate_gatt_connection_response(bidi_session, top_context,
        test_page, subscribe_events, wait_for_event, code):
    device_address = await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event)
    await subscribe_events(
        events=[BLUETOOTH_GATT_CONNECTION_ATTEMPTED_EVENT])

    # Schedule device gatt connect via WEB API. It will be blocked on the gatt response simulation
    # and resolved after the gatt response code is simulated.
    gatt_connect_future = asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration="""async ()=>{
                const devices = await navigator.bluetooth.getDevices();
                const device = devices[0];
                try {
                  await device.gatt.connect();
                } finally {
                  return device.gatt.connected;
                }
            }
            """,
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))

    gatt_connection_attempted_event = await wait_for_event(
        BLUETOOTH_GATT_CONNECTION_ATTEMPTED_EVENT)
    recursive_compare({
        "context": top_context["context"],
        "address": device_address,
    }, gatt_connection_attempted_event)

    await bidi_session.bluetooth.simulate_gatt_connection_response(
        context=top_context["context"],
        address=device_address, code=code)

    gatt_connect = await gatt_connect_future
    recursive_compare({
        "type": "boolean",
        "value": True if code == 0x0 else False
    }, gatt_connect)
