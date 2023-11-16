import pytest
from webdriver.bidi.modules.script import ContextTarget

from . import assert_console_entry, create_console_api_message_from_string
from ... import any_string, int_interval


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "log_argument, expected_text",
    [
        ("'TEST'", "TEST"),
        ("'TWO', 'PARAMETERS'", "TWO PARAMETERS"),
        ("{}", any_string),
        ("['1', '2', '3']", any_string),
        ("null, undefined", "null undefined"),
    ],
    ids=[
        "single string",
        "two strings",
        "empty object",
        "array of strings",
        "null and undefined",
    ],
)
async def test_text_with_argument_variation(
    bidi_session, subscribe_events, top_context, wait_for_event, log_argument, expected_text,
):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, "log", log_argument)
    event_data = await on_entry_added

    assert_console_entry(event_data, text=expected_text, context=top_context["context"])


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "log_method, expected_level",
    [
        ("assert", "error"),
        ("debug", "debug"),
        ("error", "error"),
        ("info", "info"),
        ("log", "info"),
        ("table", "info"),
        ("trace", "debug"),
        ("warn", "warn"),
    ],
)
async def test_level(
    bidi_session, subscribe_events, top_context, wait_for_event, log_method, expected_level
):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    if log_method == "assert":
        # assert has to be called with a first falsy argument to trigger a log.
        await create_console_api_message_from_string(
            bidi_session, top_context, "assert", "false, 'foo'")
    else:
        await create_console_api_message_from_string(
            bidi_session, top_context, log_method, "'foo'")

    event_data = await on_entry_added

    assert_console_entry(
        event_data, text="foo", level=expected_level, method=log_method
    )


@pytest.mark.asyncio
async def test_timestamp(bidi_session, subscribe_events, top_context, wait_for_event, current_time):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    time_start = await current_time()

    script = """new Promise(resolve => {
            setTimeout(() => {
                console.log('foo');
                resolve();
            }, 100);
        });
        """
    await bidi_session.script.evaluate(
        expression=script,
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await on_entry_added

    time_end = await current_time()

    assert_console_entry(event_data, text="foo", timestamp=int_interval(time_start, time_end))


@pytest.mark.asyncio
async def test_new_context_with_new_window(bidi_session, subscribe_events, top_context, wait_for_event):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, 'log', "'foo'")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="foo", context=top_context["context"])

    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, new_context, 'log', "'foo_in_new_window'")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="foo_in_new_window", context=new_context["context"])


@pytest.mark.asyncio
async def test_new_context_with_refresh(bidi_session, subscribe_events, top_context, wait_for_event):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, 'log', "'foo'")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="foo", context=top_context["context"])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=top_context["url"], wait="complete"
    )
    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, 'log', "'foo_after_refresh'")
    event_data = await on_entry_added
    assert_console_entry(
        event_data, text="foo_after_refresh", context=top_context["context"]
    )


@pytest.mark.asyncio
async def test_different_contexts(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    test_page_same_origin_frame,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_same_origin_frame, wait="complete"
    )
    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    assert len(contexts[0]["children"]) == 1
    frame_context = contexts[0]["children"][0]

    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, "log", "'foo'")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="foo", context=top_context["context"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, frame_context, "log", "'bar'")
    event_data = await on_entry_added
    assert_console_entry(event_data, text="bar", context=frame_context["context"])
