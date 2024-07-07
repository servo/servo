import pytest_asyncio

from .. import RESPONSE_COMPLETED_EVENT


@pytest_asyncio.fixture
async def is_request_from_cache(
    wait_for_event, fetch, wait_for_future_safe, top_context
):
    async def is_request_from_cache(url, context=top_context):
        on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
        await fetch(url, context=context)
        event = await wait_for_future_safe(on_response_completed)

        return event["response"]["fromCache"]

    return is_request_from_cache
