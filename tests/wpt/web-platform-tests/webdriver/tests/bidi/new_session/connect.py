import pytest
import asyncio
import websockets

# classic session to enable bidi capability manually
@pytest.mark.asyncio
@pytest.mark.capabilities({"webSocketUrl": True})
async def test_websocket_url_connect(session):
    websocket_url = session.capabilities["webSocketUrl"]
    async with websockets.connect(websocket_url) as websocket:
        await websocket.send("Hello world!")
        await websocket.close()

# bidi session following classic session to test session
# recreation with bidi true in session fixture.
# Close websocket at the end.
@pytest.mark.asyncio
@pytest.mark.bidi(True)
async def test_bidi_session_1(session):
    await session.websocket_transport.send("test_bidi_session_1")
    await session.websocket_transport.close()

# bidi session following a bidi session with the same capabilities
# but closed websocket to test restart of websocket connection.
@pytest.mark.asyncio
@pytest.mark.bidi(True)
async def test_bidi_session_2(session):
    await session.websocket_transport.send("test_bidi_session_2")
    await session.websocket_transport.close()

# bidi session following a bidi session with a different capabilities
# to test session recreation
@pytest.mark.asyncio
@pytest.mark.bidi(True)
@pytest.mark.capabilities({"acceptInsecureCerts": True})
async def test_bidi_session_3(session):
    await session.websocket_transport.send("test_bidi_session_3")

# classic session following a bidi session to test session
# recreation
@pytest.mark.asyncio
async def test_classic(session):
    pass
