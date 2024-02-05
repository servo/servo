import pytest_asyncio

from tests.support.sync import AsyncPoll

# Temporary fixtures until traverse history is fully implemented and will await the navigation.
# See: https://github.com/w3c/webdriver-bidi/issues/94


@pytest_asyncio.fixture
async def wait_for_url(bidi_session, current_url):
    async def wait_for_url(context, target_url, timeout=2):
        async def check_url(_):
            return await current_url(context) == target_url

        wait = AsyncPoll(
            bidi_session,
            timeout=timeout,
            message="Expected URL did not load"
        )
        await wait.until(check_url)

    return wait_for_url


@pytest_asyncio.fixture
async def wait_for_not_url(bidi_session, current_url):
    async def wait_for_not_url(context, target_url, timeout=2):
        async def check_url(_):
            return await current_url(context) != target_url

        wait = AsyncPoll(
            bidi_session,
            timeout=timeout,
            message="Expected URL is still loaded"
        )
        await wait.until(check_url)

    return wait_for_not_url
