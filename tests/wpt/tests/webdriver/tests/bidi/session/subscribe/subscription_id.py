import pytest
import uuid

@pytest.mark.asyncio
async def test_subscribe_subscription_id(subscribe_events):
    result = await subscribe_events(events=["browsingContext"])
    assert isinstance(result['subscription'], str)
    uuid.UUID(hex=result['subscription'])
