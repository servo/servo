import pytest_asyncio
import random

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


@pytest_asyncio.fixture
async def is_cache_enabled_for_context(fetch, top_context, url, is_request_from_cache):
    async def is_cache_enabled_for_context(context=top_context):
        cached_url = url(
            f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
        )

        # Make first request to fill up the cache.
        await is_request_from_cache(url=cached_url, context=context)

        return await is_request_from_cache(url=cached_url, context=context)

    return is_cache_enabled_for_context
