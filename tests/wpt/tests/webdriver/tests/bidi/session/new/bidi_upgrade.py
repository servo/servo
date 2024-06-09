import pytest
import websockets

import webdriver

pytestmark = pytest.mark.asyncio


# WebDriver HTTP session with BiDi upgrade path. Intended to be the first
# test in this file.
@pytest.mark.capabilities({"webSocketUrl": True})
async def test_websocket_url_connect(session):
    websocket_url = session.capabilities["webSocketUrl"]
    async with websockets.connect(websocket_url) as websocket:
        await websocket.send("Hello world!")


# Test bidi_session fixture to send a command.
async def test_bidi_session(bidi_session):
    await bidi_session.session.status()


# Test bidi_session fixture for session recreation.
@pytest.mark.capabilities({"acceptInsecureCerts": True})
async def test_bidi_session_with_different_capability(bidi_session):
    await bidi_session.session.status()


# Test session fixture following an upgraded BiDi session to test session
# recreation without BiDi upgrade. Intended to be the last test in this file
# to make sure HTTP-only session is not impacted by BiDi tests.
async def test_classic_after_bidi_session(session):
    assert not isinstance(session, webdriver.bidi.BidiSession)
