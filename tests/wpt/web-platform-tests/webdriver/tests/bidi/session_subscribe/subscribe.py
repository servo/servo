import asyncio

import pytest

from webdriver.bidi.error import InvalidArgumentException


@pytest.mark.asyncio
async def test_params_empty(bidi_session, send_blocking_command):
    with pytest.raises(InvalidArgumentException):
        response = await send_blocking_command("session.subscribe", {})


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, "foo", 42, {}])
async def test_params_events_invalid_type(bidi_session, send_blocking_command, value):
    with pytest.raises(InvalidArgumentException):
        response = await send_blocking_command("session.subscribe", {"events": value})


@pytest.mark.asyncio
async def test_params_events_empty(bidi_session):
    response = await bidi_session.session.subscribe(events=[])
    assert response == {}


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_events_value_invalid_type(send_blocking_command, value):
    with pytest.raises(InvalidArgumentException):
        response = await send_blocking_command("session.subscribe", {"events": [value]})


@pytest.mark.asyncio
@pytest.mark.parametrize("value", ["", "foo", "foo.bar"])
async def test_params_events_value_invalid_event_name(send_blocking_command, value):
    with pytest.raises(InvalidArgumentException):
        response = await send_blocking_command("session.subscribe", {"events": [value]})


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, "foo", 42, {}])
async def test_params_contexts_invalid_type(bidi_session, send_blocking_command, value):
    with pytest.raises(InvalidArgumentException):
        response = await send_blocking_command(
            "session.subscribe",
            {
                "events": [],
                "contexts": value,
            }
        )


@pytest.mark.asyncio
async def test_params_contexts_empty(bidi_session):
    response = await bidi_session.session.subscribe(events=[], contexts=[])
    assert response == {}


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_contexts_value_invalid_type(send_blocking_command, value):
    with pytest.raises(InvalidArgumentException):
        response = await send_blocking_command(
            "session.subscribe",
            {
                "events": [],
                "contexts": [value],
            }
        )
