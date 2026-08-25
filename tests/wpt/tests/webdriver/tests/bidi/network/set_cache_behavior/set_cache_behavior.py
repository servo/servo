import pytest

from .. import RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_set_cache_behavior(
    bidi_session, setup_network_test, url, is_cache_enabled_for_context
):
    await setup_network_test(events=[RESPONSE_COMPLETED_EVENT])

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context() is True

    await bidi_session.network.set_cache_behavior(cache_behavior="bypass")

    assert await is_cache_enabled_for_context() is False

    await bidi_session.network.set_cache_behavior(cache_behavior="default")

    assert await is_cache_enabled_for_context() is True


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(
    bidi_session, setup_network_test, inline, is_cache_enabled_for_context, type_hint
):
    await setup_network_test(events=[RESPONSE_COMPLETED_EVENT])

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context() is True

    await bidi_session.network.set_cache_behavior(cache_behavior="bypass")

    assert await is_cache_enabled_for_context() is False

    # Create a new tab.
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<div>foo</div>"),
        wait="complete",
    )

    # Make sure that the new context still has cache disabled.
    assert await is_cache_enabled_for_context(new_context) is False

    # Reset to default behavior.
    await bidi_session.network.set_cache_behavior(cache_behavior="default")
