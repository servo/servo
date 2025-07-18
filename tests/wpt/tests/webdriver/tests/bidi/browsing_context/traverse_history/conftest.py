import pytest_asyncio

from tests.support.sync import AsyncPoll

# Temporary fixtures until traverse history is fully implemented and will await the navigation.
# See: https://github.com/w3c/webdriver-bidi/issues/94


@pytest_asyncio.fixture
async def wait_for_url(bidi_session, current_url):
    async def wait_for_url(context, target_url, timeout=2):
        async def check_url(_):
            assert await current_url(context) == target_url, "Expected URL did not load"

        wait = AsyncPoll(bidi_session, timeout=timeout)
        await wait.until(check_url)

    return wait_for_url


@pytest_asyncio.fixture
async def wait_for_not_url(bidi_session, current_url):
    async def wait_for_not_url(context, target_url, timeout=2):
        async def check_url_different(_):
            assert await current_url(context) != target_url, "Expected URL is still loaded"

        wait = AsyncPoll(bidi_session, timeout=timeout)
        await wait.until(check_url_different)

    return wait_for_not_url
