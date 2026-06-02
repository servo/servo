import pytest

from tests.bidi import recursive_compare
from webdriver.bidi.modules.script import ContextTarget
from . import OFFLINE_NETWORK_CONDITIONS

pytestmark = pytest.mark.asyncio


async def test_navigator_online(bidi_session, top_context,
        get_navigator_online):
    assert await get_navigator_online(top_context)
    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        contexts=[top_context["context"]])
    assert not await get_navigator_online(top_context)
    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        contexts=[top_context["context"]])
    assert await get_navigator_online(top_context)


async def test_fetch(bidi_session, top_context, url, get_can_fetch):
    # Navigate away from about:blank to allow fetch requests.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url(f"/common/blank.html"),
        wait="complete")

    assert await get_can_fetch(top_context)
    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        contexts=[top_context["context"]])
    assert not await get_can_fetch(top_context)
    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        contexts=[top_context["context"]])
    assert await get_can_fetch(top_context)


async def test_navigate(bidi_session, top_context, url,
        get_can_navigate):
    assert await get_can_navigate(top_context)
    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        contexts=[top_context["context"]])
    assert not await get_can_navigate(top_context)
    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        contexts=[top_context["context"]])
    assert await get_can_navigate(top_context)


async def test_window_offline_online_events(bidi_session, top_context, url,
        subscribe_events, wait_for_event, wait_for_future_safe):
    await subscribe_events(["script.message"])
    await bidi_session.script.call_function(
        function_declaration="""(channel)=>{
            window.addEventListener("offline", (e) => {{
                channel("offline, isTrusted: "+ e.isTrusted);
            }});
            window.addEventListener("online", (e) => {{
                channel("online, isTrusted: "+ e.isTrusted);
            }});
        }""",
        arguments=[{
            "type": "channel",
            "value": {
                "channel": "channel_name"
            }
        }],
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    on_script_message = wait_for_event("script.message")
    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        contexts=[top_context["context"]])

    # Wait for the window `offline` event.
    event_data = await wait_for_future_safe(on_script_message)
    recursive_compare(
        {
            'channel': 'channel_name',
            'data': {
                'type': 'string',
                'value': 'offline, isTrusted: true'
            },
        }, event_data,
    )

    on_script_message = wait_for_event("script.message")
    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        contexts=[top_context["context"]])

    # Wait for the window `online` event.
    event_data = await wait_for_future_safe(on_script_message)
    recursive_compare(
        {
            'channel': 'channel_name',
            'data': {
                'type': 'string',
                'value': 'online, isTrusted: true'
            },
        }, event_data,
    )


@pytest.mark.skip("TODO: implement the test")
async def test_websocket_disconnected():
    pass


@pytest.mark.skip("TODO: implement the test")
async def test_service_worker_fetch():
    pass


@pytest.mark.skip("TODO: implement the test")
async def test_service_worker_websocket_disconnected():
    pass
