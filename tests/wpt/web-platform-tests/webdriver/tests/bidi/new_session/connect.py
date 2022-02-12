import pytest
import websockets

import webdriver

# classic session to enable bidi capability manually
# Intended to be the first test in this file
@pytest.mark.asyncio
@pytest.mark.capabilities({"webSocketUrl": True})
async def test_websocket_url_connect(session):
    websocket_url = session.capabilities["webSocketUrl"]
    async with websockets.connect(websocket_url) as websocket:
        await websocket.send("Hello world!")

# test bidi_session send
@pytest.mark.asyncio
async def test_bidi_session_send(bidi_session, send_blocking_command):
    await send_blocking_command("session.status", {})

# bidi session following a bidi session with a different capabilities
# to test session recreation
@pytest.mark.asyncio
@pytest.mark.capabilities({"acceptInsecureCerts": True})
async def test_bidi_session_with_different_capability(bidi_session,
                                                      send_blocking_command):
    await send_blocking_command("session.status", {})

# classic session following a bidi session to test session
# recreation
# Intended to be the last test in this file to make sure
# classic session is not impacted by bidi tests
@pytest.mark.asyncio
def test_classic_after_bidi_session(session):
    assert not isinstance(session, webdriver.bidi.BidiSession)
