import pytest
import random

from .. import RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_set_cache_bypass(
    bidi_session, setup_network_test, url, is_request_from_cache
):
    await setup_network_test(events=[RESPONSE_COMPLETED_EVENT])

    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
    )

    # The first request/response is used to fill the browser cache,
    # so we expect fromCache to be False here.
    assert await is_request_from_cache(cached_url) is False

    # The second request for the same URL has to be read from the local cache.
    assert await is_request_from_cache(cached_url) is True

    await bidi_session.network.set_cache_bypass(bypass=True)

    assert await is_request_from_cache(cached_url) is False

    await bidi_session.network.set_cache_bypass(bypass=False)

    assert await is_request_from_cache(cached_url) is True


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(
    bidi_session, setup_network_test, url, inline, is_request_from_cache, type_hint
):
    await setup_network_test(events=[RESPONSE_COMPLETED_EVENT])

    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
    )

    # The first request/response is used to fill the browser cache,
    # so we expect fromCache to be False here.
    assert await is_request_from_cache(cached_url) is False

    # The second request for the same URL has to be read from the local cache.
    assert await is_request_from_cache(cached_url) is True

    await bidi_session.network.set_cache_bypass(bypass=True)

    assert await is_request_from_cache(cached_url) is False

    # Create a new tab.
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<div>foo</div>"),
        wait="complete",
    )

    # Make sure that the new context still has cache disabled.
    assert await is_request_from_cache(cached_url, context=new_context) is False

    # Reset to default behavior.
    await bidi_session.network.set_cache_bypass(bypass=False)
