import re

import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


# This test suite is tentative since in these cases
# text field can contain implementation-defined messages.


LOG_ENTRY_ADDED = "log.entryAdded"
TIMESTAMP_REGEX = r"(\d+(\.|,)?\d*) ?ms"


@pytest.mark.parametrize(
    "script, message",
    [
        ('console.count("test")', "test: 1"),
        ("console.count()", "default: 1"),
        ("console.log(undefined)", "undefined"),
        ("console.log(null)", "null"),
        ('console.log("null")', "null"),
        ("console.log([1])", "Array(1)"),
        ("console.log({a: 1})", "Object(1)"),
        ("console.log(new Set([1]))", "Set(1)"),
        ('console.log(new Map([["a", 1]]))', "Map(1)"),
    ],
)
async def test_text(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    script,
    message,
):
    await subscribe_events(events=[LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)

    await bidi_session.script.evaluate(
        expression=script,
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_entry_added)

    assert event_data["text"] == message
    assert event_data["level"] == "info"


@pytest.mark.parametrize(
    "script, message",
    [
        ("console.time(); console.time();", "default"),
        ('console.timeEnd("nonexistent")', "nonexistent"),
        ('console.timeLog("nonexistent")', "nonexistent"),
        ('console.countReset("nonexistent")', "nonexistent"),
    ],
)
async def test_text_warn(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    script,
    message,
):
    await subscribe_events(events=[LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)

    await bidi_session.script.evaluate(
        expression=script,
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_entry_added)

    assert message in event_data["text"]
    assert event_data["level"] == "warn"


async def test_text_time(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events(events=[LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)

    time_message_regex = r"test1: " + TIMESTAMP_REGEX

    await bidi_session.script.evaluate(
        expression="""console.time("test1"); console.timeLog("test1");""",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_entry_added)

    assert re.match(time_message_regex, event_data["text"])
    assert event_data["level"] == "info"

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)

    await bidi_session.script.evaluate(
        expression="""console.timeEnd("test1");""",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_entry_added)

    assert re.match(time_message_regex, event_data["text"])
    assert event_data["level"] == "info"


async def test_text_timelog_with_extra_args(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events(events=[LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)

    await bidi_session.script.evaluate(
        expression="""console.time("test2"); console.timeLog("test2", "foo", [1]);""",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_entry_added)

    assert re.match(
        r"test2: " + TIMESTAMP_REGEX + r" foo Array\(1\)", event_data["text"]
    )
    assert event_data["level"] == "info"

@pytest.mark.parametrize(
    "script, message",
    [
        ("console.log(new Error('err'))", "Error: err"),
        ("console.log(new TypeError('t'))", "TypeError: t"),
        ("console.log(new SyntaxError('s'))", "SyntaxError: s"),
    ],
)
async def test_text_error(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    script,
    message,
):
    await subscribe_events(events=[LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)

    await bidi_session.script.evaluate(
        expression=script,
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await wait_for_future_safe(on_entry_added)

    if bidi_session.capabilities["browserName"] == "firefox":
        assert event_data["text"] == message
    else:
        assert event_data["text"] == "error"
    assert event_data["level"] == "info"
