import json
import asyncio

from webdriver.bidi.modules.script import ContextTarget

TEST_DEVICE_NAME = 'SOME_BL_DEVICE'
TEST_DEVICE_ADDRESS = '09:09:09:09:09:09'
TEST_DEVICE_2_ADDRESS = '10:10:10:10:10:10'
BLUETOOTH_REQUEST_DEVICE_PROMPT_UPDATED_EVENT = 'bluetooth.requestDevicePromptUpdated'
BLUETOOTH_GATT_CONNECTION_ATTEMPTED_EVENT = 'bluetooth.gattConnectionAttempted'
BLUETOOTH_CHARACTERISTIC_EVENT_GENERATED_EVENT = 'bluetooth.characteristicEventGenerated'
BLUETOOTH_DESCRIPTOR_EVENT_GENERATED_EVENT = 'bluetooth.descriptorEventGenerated'
HEART_RATE_SERVICE_UUID = '0000180d-0000-1000-8000-00805f9b34fb'
BATTERY_SERVICE_UUID = '0000180f-0000-1000-8000-00805f9b34fb'
MEASUREMENT_INTERVAL_CHARACTERISTIC_UUID = '00002a21-0000-1000-8000-00805f9b34fb'
DATE_TIME_CHARACTERISTIC_UUID = '00002a08-0000-1000-8000-00805f9b34fb'
CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID = '00002901-0000-1000-8000-00805f9b34fb'
CLIENT_CHARACTERISTIC_CONFIGURATION_DESCRIPTOR_UUID = '00002902-0000-1000-8000-00805f9b34fb'


async def set_simulate_adapter(bidi_session, context, test_page, state):
    # Navigate to a page, as bluetooth is not guaranteed to work on
    # `about:blank`.
    await bidi_session.browsing_context.navigate(context=context['context'],
                                                 url=test_page, wait="complete")

    await bidi_session.bluetooth.simulate_adapter(context=context["context"],
                                                  state=state)


async def set_simulate_preconnected_peripheral(bidi_session, context, test_page,
                                               address, name, manufacturer_data,
                                               known_service_uuids):
    # Navigate to a page, as bluetooth is not guaranteed to work on
    # `about:blank`.
    await bidi_session.browsing_context.navigate(context=context['context'],
                                                 url=test_page, wait="complete")
    await bidi_session.bluetooth.simulate_adapter(context=context["context"],
                                                  state="powered-on")
    await bidi_session.bluetooth.simulate_preconnected_peripheral(
        context=context["context"],
        address=address, name=name,
        manufacturer_data=manufacturer_data,
        known_service_uuids=known_service_uuids)


def request_device(context, bidi_session, optional_services= []):
    return asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration=f"""async (device_name)=>{{
                const device = await navigator.bluetooth.requestDevice({{
                    filters: [{{name:device_name}}],
                    optionalServices: {json.dumps(optional_services)}
                }});
                return {{
                    id: device.id,
                    name: device.name,
                }}
            }}
            """,
            arguments=[{"type": "string", "value": TEST_DEVICE_NAME}],
            target=ContextTarget(context["context"]),
            await_promise=True,
            # Required to emulate user activated the request.
            user_activation=True
        ))


async def setup_granted_device(bidi_session, context, test_page, subscribe_events, wait_for_event, optional_services = []):
    await set_simulate_preconnected_peripheral(
        bidi_session,
        context,
        test_page,
        TEST_DEVICE_ADDRESS,
        TEST_DEVICE_NAME,
        [{"key": 17, "data": "AP8BAX8="}],
        ["12345678-1234-5678-9abc-def123456789"],
    )

    await subscribe_events(
        events=[BLUETOOTH_REQUEST_DEVICE_PROMPT_UPDATED_EVENT])

    # Set prompt listener.
    bluetooth_prompt_future = wait_for_event(
        BLUETOOTH_REQUEST_DEVICE_PROMPT_UPDATED_EVENT)

    # Schedule requesting device via WEB API. It will be blocked on the prompt
    # and resolved after the prompt is addressed.
    request_device_future = request_device(context, bidi_session, optional_services)

    # Wait for the prompt.
    bluetooth_prompt = await bluetooth_prompt_future

    # Accept the prompt.
    await bidi_session.bluetooth.handle_request_device_prompt(
        context=context["context"],
        prompt=bluetooth_prompt["prompt"],
        accept=True,
        device=bluetooth_prompt['devices'][0]['id']
    )

    # Wait for the script to finish.
    await request_device_future
    return TEST_DEVICE_ADDRESS


async def create_gatt_connection(bidi_session, context, subscribe_events, wait_for_event):
    await subscribe_events(
        events=[BLUETOOTH_GATT_CONNECTION_ATTEMPTED_EVENT])
    gatt_connect_future = asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration="""async ()=>{
                const devices = await navigator.bluetooth.getDevices();
                const device = devices[0];
                await device.gatt.connect();
            }
            """,
            target=ContextTarget(context["context"]),
            await_promise=True,
        ))

    gatt_connection_attempted_event = await wait_for_event(
        BLUETOOTH_GATT_CONNECTION_ATTEMPTED_EVENT)

    await bidi_session.bluetooth.simulate_gatt_connection_response(
        context=context["context"],
        address=gatt_connection_attempted_event["address"], code=0x0)
    await gatt_connect_future


def remote_array_to_list(remote_array) -> list:
    assert remote_array["type"] == "array"
    return [item["value"] for item in remote_array["value"]]