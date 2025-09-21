import pytest

from webdriver.bidi.error import InvalidArgumentException

from ... import create_console_api_message


@pytest.mark.asyncio
async def test_params_empty(send_blocking_command):
    with pytest.raises(InvalidArgumentException):
        await send_blocking_command("session.unsubscribe", {})


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, "foo", 42, {}])
async def test_params_events_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=value)


@pytest.mark.asyncio
async def test_params_events_empty(bidi_session):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=[])


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_events_value_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=[value])


@pytest.mark.asyncio
@pytest.mark.parametrize("value", ["", "foo", "foo.bar"])
async def test_params_events_value_invalid_event_name(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=[value])


@pytest.mark.asyncio
async def test_params_events_value_valid_and_invalid_event_name(
    bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, top_context
):
    # Subscribe to a valid event
    await subscribe_events(events=["log.entryAdded"])

    # Try to unsubscribe from the valid and an invalid event
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=[
            "log.entryAdded", "some.invalidEvent"])

    # Make sure that we didn't unsubscribe from log.entryAdded because of the error
    # and events are still coming

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        "log.entryAdded", on_event)

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message(bidi_session, top_context, "text1")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_unsubscribe_from_one_event_and_then_from_module(
    bidi_session, subscribe_events
):
    await subscribe_events(events=["browsingContext"])

    # Unsubscribe from one event
    await bidi_session.session.unsubscribe(events=["browsingContext.domContentLoaded"])

    # Try to unsubscribe from all events
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=["browsingContext"])

    # Unsubscribe from the rest of the events
    await bidi_session.session.unsubscribe(events=["browsingContext.contextCreated"])
    await bidi_session.session.unsubscribe(events=["browsingContext.load"])


@pytest.mark.asyncio
async def test_params_unsubscribe_globally_without_subscription(bidi_session):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=["log.entryAdded"])


@pytest.mark.asyncio
async def test_params_unsubscribe_globally_with_individual_subscription(
    subscribe_events, bidi_session, top_context
):
    # Subscribe to one context
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])

    # Try to unsubscribe globally
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(events=["log.entryAdded"])


@pytest.mark.asyncio
@pytest.mark.parametrize("subscriptions", [None, True, 42, {}, "foo"])
async def test_params_subscriptions_invalid_type(bidi_session, subscriptions):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(subscriptions=subscriptions)


@pytest.mark.asyncio
@pytest.mark.parametrize("subscription", [None, True, 42, {}, []])
async def test_params_subscriptions_entry_invalid_type(bidi_session, subscription):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(subscriptions=[subscription])


@pytest.mark.asyncio
@pytest.mark.parametrize("subscriptions", [[""], ["12345678-1234-5678-1234-567812345678"]])
async def test_params_subscriptions_invalid_value(bidi_session, subscriptions):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(subscriptions=subscriptions)


@pytest.mark.asyncio
async def test_unsubscribe_with_subscription_id_twice(bidi_session):
    result = await bidi_session.session.subscribe(events=["log.entryAdded"])

    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])

    # Trying to unsubscribe second time with the same subscription id should fail.
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])
