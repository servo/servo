import pytest
from webdriver.bidi.modules.script import ContextTarget

from ... import any_string, recursive_compare


pytestmark = pytest.mark.asyncio


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
                    "serializationOptions": {"maxObjectDepth": 0},
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
    bidi_session,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    add_preload_script,
    channel,
    expected_data,
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    await add_preload_script(
        function_declaration="""(channel) => channel({'foo': 'bar', 'baz': {'1': 2}})""",
        arguments=[channel],
    )

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    event_data = await wait_for_future_safe(on_script_message)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": expected_data,
            "source": {
                "realm": any_string,
                "context": new_tab["context"],
            },
        },
        event_data,
    )


async def test_channel_with_multiple_arguments(
    bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, add_preload_script
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    await add_preload_script(
        function_declaration="""(channel) => channel('will_be_send', 'will_be_ignored')""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
    )

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    event_data = await wait_for_future_safe(on_script_message)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {"type": "string", "value": "will_be_send"},
            "source": {
                "realm": any_string,
                "context": new_tab["context"],
            },
        },
        event_data,
    )


async def test_mutation_observer(
    bidi_session,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    new_tab,
    inline,
    add_preload_script,
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    await add_preload_script(
        function_declaration="""(channel) => {
            const onMutation = (mutationList) => mutationList.forEach(mutation => {
                const attributeName = mutation.attributeName;
                const newValue = mutation.target.getAttribute(mutation.attributeName);
                channel({ attributeName, newValue });
            });
            const observer = new MutationObserver(onMutation);
            observer.observe(document, { attributes: true, subtree: true });
        }""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
    )

    url = inline("<div class='old class name'>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete",
    )

    restult = await bidi_session.script.evaluate(
        raw_result=True,
        expression="document.querySelector('div').setAttribute('class', 'mutated')",
        await_promise=True,
        target=ContextTarget(new_tab["context"]),
    )

    event_data = await wait_for_future_safe(on_script_message)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {
                "type": "object",
                "value": [
                    ["attributeName", {"type": "string", "value": "class"}],
                    ["newValue", {"type": "string", "value": "mutated"}],
                ],
            },
            "source": {
                "realm": restult["realm"],
                "context": new_tab["context"],
            },
        },
        event_data,
    )


async def test_two_channels(
    bidi_session,
    subscribe_events,
    wait_for_bidi_events,
    add_preload_script,
):
    await subscribe_events(["script.message"])

    # Track all received script.message events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("script.message", on_event)

    await add_preload_script(
        function_declaration="""(channel_1, channel_2) => {
            channel_1('message_from_channel_1');
            channel_2('message_from_channel_2')
        }""",
        arguments=[
            {"type": "channel", "value": {"channel": "channel_name_1"}},
            {"type": "channel", "value": {"channel": "channel_name_2"}},
        ],
    )

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    # Wait for both events
    await wait_for_bidi_events(events, 2, timeout=0.5)

    recursive_compare(
        {
            "channel": "channel_name_1",
            "data": {"type": "string", "value": "message_from_channel_1"},
            "source": {
                "realm": any_string,
                "context": new_tab["context"],
            },
        },
        events[0],
    )

    recursive_compare(
        {
            "channel": "channel_name_2",
            "data": {"type": "string", "value": "message_from_channel_2"},
            "source": {
                "realm": any_string,
                "context": new_tab["context"],
            },
        },
        events[1],
    )

    remove_listener()


async def test_channel_with_window_open(
    bidi_session,
    subscribe_events,
    new_tab,
    inline,
    wait_for_event,
    wait_for_bidi_events,
    wait_for_future_safe,
    add_preload_script,
):
    await subscribe_events(["script.message", "browsingContext.contextCreated"])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(f"""<button id="open" onclick='window.open("{inline('<div>popup</div>')}")'>Open popup</button>"""),
        wait="complete",
    )

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("script.message", on_event)

    on_created = wait_for_event("browsingContext.contextCreated")

    await add_preload_script(
        function_declaration="""(channel) => channel({'script': 1})""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
    )

    await add_preload_script(
        function_declaration="""(channel) => channel({'script': 2})""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
    )

    # Trigger "window.open" by clicking on the button.
    await bidi_session.script.evaluate(
        expression="document.getElementById('open').click()",
        target={"context": new_tab["context"]},
        await_promise=True,
    )

    # Firefox might receive more events because of an interim realm.
    await wait_for_bidi_events(events, 2, timeout=0.5, equal_check=False)

    on_created_event_data = await wait_for_future_safe(on_created)

    for index in range(2):
        recursive_compare(
            {
                "channel": "channel_name",
                "data": {
                    "type": "object",
                    "value": [
                        ["script", {"type": "number", "value": index + 1}],
                    ],
                },
                "source": {
                    "realm": any_string,
                    "context": on_created_event_data["context"],
                },
            },
            events[index],
        )

    remove_listener()


async def test_channel_with_window_open_subscribe_to_user_context(
    bidi_session,
    subscribe_events,
    new_tab,
    inline,
    wait_for_event,
    wait_for_future_safe,
    add_preload_script,
):
    await subscribe_events(["browsingContext.contextCreated"])

    # Subscribe to only a default user context.
    await subscribe_events(["script.message"], user_contexts=["default"])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(f"""<button id="open" onclick='window.open("{inline('<div>popup</div>')}")'>Open popup</button>"""),
        wait="complete",
    )

    on_created = wait_for_event("browsingContext.contextCreated")
    on_message = wait_for_event("script.message")

    await add_preload_script(
        function_declaration="""(channel) => channel({'script': 1})""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
    )

    # Trigger "window.open" by clicking on the button.
    await bidi_session.script.evaluate(
        expression="document.getElementById('open').click()",
        target={"context": new_tab["context"]},
        await_promise=True,
    )

    message_event_data = await wait_for_future_safe(on_message)
    created_event_data = await wait_for_future_safe(on_created)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {
                "type": "object",
                "value": [
                    ["script", {"type": "number", "value": 1}],
                ],
            },
            "source": {
                "realm": any_string,
                "context": created_event_data["context"],
            },
        },
        message_event_data,
    )
