import pytest
from webdriver.error import TimeoutException

from tests.support.sync import AsyncPoll


# The basic use case of unsubscribing globally from a single event
# is covered by tests for each event in the dedicated folders.


@pytest.mark.asyncio
async def test_unsubscribe_from_module(bidi_session):
    await bidi_session.session.subscribe(events=["browsingContext"])
    await bidi_session.session.unsubscribe(events=["browsingContext"])

    # Track all received browsing context events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener_contextCreated = bidi_session.add_event_listener(
        "browsingContext.contextCreated", on_event
    )
    remove_listener_domContentLoaded = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_load = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    await bidi_session.browsing_context.create(type_hint="tab")

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener_contextCreated()
    remove_listener_domContentLoaded()
    remove_listener_load()


@pytest.mark.asyncio
async def test_subscribe_to_module_unsubscribe_from_one_event(
    bidi_session, wait_for_event, wait_for_future_safe
):
    await bidi_session.session.subscribe(events=["browsingContext"])

    # Unsubscribe from one event
    await bidi_session.session.unsubscribe(events=["browsingContext.domContentLoaded"])

    # Track all received browsing context events in the events array
    events = []

    async def on_event(method, _):
        events.append(method)

    remove_listener_contextCreated = bidi_session.add_event_listener(
        "browsingContext.contextCreated", on_event
    )
    remove_listener_domContentLoaded = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_load = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    # Wait for the last event
    on_entry_added = wait_for_event("browsingContext.load")
    await bidi_session.browsing_context.create(type_hint="tab")
    await wait_for_future_safe(on_entry_added)

    # Make sure we didn't receive browsingContext.domContentLoaded event
    assert len(events) == 2
    assert "browsingContext.domContentLoaded" not in events

    remove_listener_contextCreated()
    remove_listener_domContentLoaded()
    remove_listener_load()

    # Unsubscribe from the rest of the events
    await bidi_session.session.unsubscribe(events=["browsingContext.contextCreated"])
    await bidi_session.session.unsubscribe(events=["browsingContext.load"])
