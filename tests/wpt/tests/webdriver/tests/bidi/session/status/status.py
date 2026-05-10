import pytest

pytestmark = pytest.mark.asyncio


# Check that session.status can be used. The actual values for the "ready" and
# "message" properties are implementation specific.
async def test_bidi_session_status(bidi_session):
    response = await bidi_session.session.status()
    assert isinstance(response["ready"], bool)
    assert isinstance(response["message"], str)
