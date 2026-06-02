import pytest, asyncio

from webdriver.bidi.modules.script import ContextTarget
from .. import setup_granted_device, create_gatt_connection
from .... import recursive_compare

pytestmark = pytest.mark.asyncio


async def test_simulate_gatt_disconnection(bidi_session, top_context,
        test_page, subscribe_events, wait_for_event):
    device_address = await setup_granted_device(bidi_session, top_context, test_page, subscribe_events, wait_for_event)
    await create_gatt_connection(bidi_session, top_context, subscribe_events, wait_for_event)

    await bidi_session.bluetooth.simulate_gatt_disconnection(
        context=top_context["context"],
        address=device_address)

    gatt_connected = await asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration="""async ()=>{
                const devices = await navigator.bluetooth.getDevices();
                const device = devices[0];
                return device.gatt.connected;
            }
            """,
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        ))
    recursive_compare({
        "type": "boolean",
        "value": False
    }, gatt_connected)
