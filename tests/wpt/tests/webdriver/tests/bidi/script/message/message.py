import pytest
from tests.support.sync import AsyncPoll

from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException


pytestmark = pytest.mark.asyncio

MESSAGE_EVENT = "script.message"


async def test_unsubscribe(bidi_session, top_context):
    await bidi_session.session.subscribe(events=[MESSAGE_EVENT])
    await bidi_session.session.unsubscribe(events=[MESSAGE_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(MESSAGE_EVENT, on_event)

    await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="(channel) => channel('foo')",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert len(events) == 0

    remove_listener()


async def test_subscribe(bidi_session, subscribe_events, top_context, wait_for_event, wait_for_future_safe):
    await subscribe_events(events=[MESSAGE_EVENT])

    on_script_message = wait_for_event(MESSAGE_EVENT)
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="(channel) => channel('foo')",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    event = await wait_for_future_safe(on_script_message)

    assert event == {
        "channel": "channel_name",
        "data": {"type": "string", "value": "foo"},
        "source": {
            "realm": result["realm"],
            "context": top_context["context"],
        },
    }


async def test_subscribe_to_one_context(
    bidi_session, subscribe_events, top_context, new_tab
):
    # Subscribe to a specific context
    await subscribe_events(
        events=[MESSAGE_EVENT], contexts=[top_context["context"]]
    )

    # Track all received script.message events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(MESSAGE_EVENT, on_event)

    # Send the event in the other context
    await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="(channel) => channel('foo')",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(new_tab["context"]),
    )

    # Make sure we didn't receive the event for the new tab
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="(channel) => channel('foo')",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    # Make sure we received the event for the right context
    assert len(events) == 1

    remove_listener()
