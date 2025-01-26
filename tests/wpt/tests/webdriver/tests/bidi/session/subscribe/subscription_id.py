import pytest
import uuid

pytestmark = pytest.mark.asyncio


async def test_subscribe_subscription_id(subscribe_events):
    result = await subscribe_events(events=["browsingContext"])
    assert isinstance(result["subscription"], str)
    uuid.UUID(hex=result["subscription"])


async def test_subscribe_twice(subscribe_events):
    result_1 = await subscribe_events(events=["browsingContext"])
    result_2 = await subscribe_events(events=["script"])

    assert result_1["subscription"] != result_2["subscription"]
