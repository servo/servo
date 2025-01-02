import asyncio

import pytest

from webdriver.bidi.error import InvalidArgumentException, NoSuchFrameException

from ... import create_console_api_message


@pytest.mark.asyncio
async def test_params_empty(send_blocking_command):
    with pytest.raises(InvalidArgumentException):
        await send_blocking_command("session.subscribe", {})


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, "foo", 42, {}])
async def test_params_events_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=value)


@pytest.mark.asyncio
async def test_params_events_empty(bidi_session):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=[])


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_events_value_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=[value])


@pytest.mark.asyncio
@pytest.mark.parametrize("value", ["", "foo", "foo.bar", "log.invalidEvent"])
async def test_params_events_value_invalid_event_name(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=[value])


@pytest.mark.asyncio
async def test_params_events_value_valid_and_invalid_event_names(
    bidi_session, top_context
):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=[
            "log.entryAdded", "some.invalidEvent"])

    # Make sure that we didn't subscribe to log.entryAdded because of the error

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        "log.entryAdded", on_event)

    await create_console_api_message(bidi_session, top_context, "text1")

    # Wait for some time before checking the events array
    await asyncio.sleep(0.5)
    assert len(events) == 0

    remove_listener()


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [True, "foo", 42, {}])
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=["log.entryAdded"], contexts=value)


@pytest.mark.asyncio
async def test_params_contexts_empty(bidi_session):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=["log.entryAdded"], contexts=[])


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_contexts_value_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.session.subscribe(events=["log.entryAdded"], contexts=[value])


@pytest.mark.asyncio
async def test_params_contexts_value_invalid_value(bidi_session):
    with pytest.raises(NoSuchFrameException):
        await bidi_session.session.subscribe(events=["log.entryAdded"], contexts=["foo"])


@pytest.mark.asyncio
async def test_params_contexts_valid_and_invalid_value(
    bidi_session, top_context
):
    with pytest.raises(NoSuchFrameException):
        await bidi_session.session.subscribe(events=["log.entryAdded"], contexts=[
            top_context["context"], "foo"])

    # Make sure that we didn't subscribe to log.entryAdded because of error

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        "log.entryAdded", on_event)

    await create_console_api_message(bidi_session, top_context, "text1")

    # Wait for some time before checking the events array
    await asyncio.sleep(0.5)
    assert len(events) == 0

    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_to_closed_tab(bidi_session):
    new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.close(context=new_tab["context"])

    # Try to subscribe to the closed context
    with pytest.raises(NoSuchFrameException):
        await bidi_session.session.subscribe(events=["log.entryAdded"], contexts=[new_tab["context"]])
