import pytest, asyncio

from webdriver.bidi.modules.script import ContextTarget
from .. import BLUETOOTH_REQUEST_DEVICE_PROMPT_UPDATED_EVENT, TEST_DEVICE_ADDRESS, TEST_DEVICE_NAME, set_simulate_preconnected_peripheral
from .... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio


async def test_simulate_preconnected_peripheral(bidi_session, top_context,
        test_page, subscribe_events, wait_for_event, wait_for_future_safe):
    await set_simulate_preconnected_peripheral(
        bidi_session,
        top_context,
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
    request_device_future = asyncio.create_task(
        bidi_session.script.call_function(
            function_declaration="""async (device_name)=>{
                const device = await navigator.bluetooth.requestDevice({
                    filters: [{name:device_name}]
                });
                return {
                    id: device.id,
                    name: device.name,
                }
            }
            """,
            arguments=[{"type": "string", "value": TEST_DEVICE_NAME}],
            target=ContextTarget(top_context["context"]),
            await_promise=True,
            # Required to emulate user activated the request.
            user_activation=True
        ))

    # Wait for the prompt.
    bluetooth_prompt = await bluetooth_prompt_future
    recursive_compare({
        "context": top_context["context"],
        "devices": [{
            "id": TEST_DEVICE_ADDRESS,
            "name": "",
        }],
        "prompt": any_string
    }, bluetooth_prompt)

    # Accept the prompt.
    await bidi_session.bluetooth.handle_request_device_prompt(
        context=top_context["context"],
        prompt=bluetooth_prompt["prompt"],
        accept=True,
        device=bluetooth_prompt['devices'][0]['id']
    )

    # Wait for the script to finish.
    requested_device = await request_device_future

    # Assert the device is expected.
    recursive_compare({
        "type": "object",
        "value": [["id", {
            "type": "string",
            "value": any_string
        }], ["name", {
            "type": "string",
            "value": TEST_DEVICE_NAME
        }]]
    }, requested_device)
