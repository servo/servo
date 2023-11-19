import pytest
from tests.support.sync import AsyncPoll
from webdriver.bidi.modules.script import ContextTarget

from ... import any_string, recursive_compare


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "channel, expected_data",
    [
        (
            {"type": "channel", "value": {"channel": "channel_name"}},
            {
                "type": "object",
                "value": [
                    ["foo", {"type": "string", "value": "bar"}],
                    [
                        "baz",
                        {
                            "type": "object",
                            "value": [["1", {"type": "number", "value": 2}]],
                        },
                    ],
                ],
            },
        ),
        (
            {
                "type": "channel",
                "value": {
                    "channel": "channel_name",
                    "serializationOptions": {
                        "maxObjectDepth": 0
                    },
                },
            },
            {"type": "object"},
        ),
        (
            {
                "type": "channel",
                "value": {"channel": "channel_name", "ownership": "root"},
            },
            {
                "handle": any_string,
                "type": "object",
                "value": [
                    ["foo", {"type": "string", "value": "bar"}],
                    [
                        "baz",
                        {
                            "type": "object",
                            "value": [["1", {"type": "number", "value": 2}]],
                        },
                    ],
                ],
            },
        ),
    ],
    ids=["default", "with serializationOptions", "with ownership"],
)
async def test_channel(
    bidi_session, top_context, subscribe_events, wait_for_event, wait_for_future_safe, channel, expected_data
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(channel) => channel({'foo': 'bar', 'baz': {'1': 2}})""",
        arguments=[channel],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    event_data = await wait_for_future_safe(on_script_message)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": expected_data,
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )


@pytest.mark.asyncio
async def test_channel_with_multiple_arguments(
    bidi_session, top_context, subscribe_events, wait_for_event, wait_for_future_safe
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(channel) => channel('will_be_send', 'will_be_ignored')""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_script_message)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {"type": "string", "value": "will_be_send"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )


@pytest.mark.asyncio
async def test_two_channels(
    bidi_session,
    top_context,
    subscribe_events,
):
    await subscribe_events(["script.message"])

    # Track all received script.message events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("script.message", on_event)

    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(channel_1, channel_2) => {
            channel_1('message_from_channel_1');
            channel_2('message_from_channel_2')
        }""",
        arguments=[
            {"type": "channel", "value": {"channel": "channel_name_1"}},
            {"type": "channel", "value": {"channel": "channel_name_2"}},
        ],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    # Wait for both events
    wait = AsyncPoll(bidi_session, timeout=0.5)
    await wait.until(lambda _: len(events) == 2)

    recursive_compare(
        {
            "channel": "channel_name_1",
            "data": {"type": "string", "value": "message_from_channel_1"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        events[0],
    )

    recursive_compare(
        {
            "channel": "channel_name_2",
            "data": {"type": "string", "value": "message_from_channel_2"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        events[1],
    )

    remove_listener()


@pytest.mark.asyncio
async def test_channel_and_nonchannel_arguments(
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    subscribe_events,
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(string, channel) => {
            channel(string);
        }""",
        arguments=[
            {"type": "string", "value": "foo"},
            {"type": "channel", "value": {"channel": "channel_name"}},
        ],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    event_data = await wait_for_future_safe(on_script_message)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {"type": "string", "value": "foo"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )
