import pytest

from .. import RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_one_context(
    bidi_session,
    setup_network_test,
    top_context,
    new_tab,
    inline,
    is_cache_enabled_for_context,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("foo"),
        wait="complete",
    )

    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT],
        contexts=[top_context["context"], new_tab["context"]],
    )

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context(top_context) is True
    assert await is_cache_enabled_for_context(new_tab) is True

    # Disable cache only in one context.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="bypass", contexts=[new_tab["context"]]
    )

    assert await is_cache_enabled_for_context(top_context) is True
    assert await is_cache_enabled_for_context(new_tab) is False

    # Reset to default behavior.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="default", contexts=[new_tab["context"]]
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(
    bidi_session,
    setup_network_test,
    top_context,
    inline,
    is_cache_enabled_for_context,
    type_hint,
):
    await setup_network_test(events=[RESPONSE_COMPLETED_EVENT])

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context() is True

    # Disable cache only in one context.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="bypass", contexts=[top_context["context"]]
    )

    assert await is_cache_enabled_for_context() is False

    # Create a new tab.
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<div>foo</div>"),
        wait="complete",
    )

    # Make sure that the new context still has cache enabled.
    assert await is_cache_enabled_for_context(context=new_context) is True

    # Reset to default behavior.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="default", contexts=[top_context["context"]]
    )


async def test_disable_globally_after_disable_for_context(
    bidi_session,
    setup_network_test,
    top_context,
    new_tab,
    inline,
    is_cache_enabled_for_context,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("foo"),
        wait="complete",
    )

    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT],
        contexts=[top_context["context"], new_tab["context"]],
    )

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context() is True
    assert await is_cache_enabled_for_context(context=new_tab) is True

    # Disable cache only in one context.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="bypass", contexts=[new_tab["context"]]
    )

    assert await is_cache_enabled_for_context() is True
    assert await is_cache_enabled_for_context(context=new_tab) is False

    # Disable cache globally.
    await bidi_session.network.set_cache_behavior(cache_behavior="bypass")

    # Make sure that cache is disabled for both contexts.
    assert await is_cache_enabled_for_context() is False
    assert await is_cache_enabled_for_context(context=new_tab) is False

    # Reset to default behavior.
    await bidi_session.network.set_cache_behavior(cache_behavior="default")


async def test_enable_globally_after_disable_for_context(
    bidi_session,
    setup_network_test,
    top_context,
    new_tab,
    inline,
    is_cache_enabled_for_context,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("foo"),
        wait="complete",
    )

    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT],
        contexts=[top_context["context"], new_tab["context"]],
    )

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context(context=top_context) is True
    assert await is_cache_enabled_for_context(context=new_tab) is True

    # Disable cache only in one context.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="bypass", contexts=[new_tab["context"]]
    )

    assert await is_cache_enabled_for_context(context=top_context) is True
    assert await is_cache_enabled_for_context(context=new_tab) is False

    # Enable cache globally.
    await bidi_session.network.set_cache_behavior(cache_behavior="default")

    # Make sure that cache is enabled for both contexts.
    assert await is_cache_enabled_for_context(context=top_context) is True
    assert await is_cache_enabled_for_context(context=new_tab) is True


async def test_setting_cache_to_contexts_after_global_update(
    bidi_session,
    setup_network_test,
    top_context,
    new_tab,
    inline,
    is_cache_enabled_for_context
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("foo"),
        wait="complete",
    )

    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT],
        contexts=[top_context["context"], new_tab["context"]],
    )

    # Make sure that cache is enabled by default.
    assert await is_cache_enabled_for_context(context=top_context) is True
    assert await is_cache_enabled_for_context(context=new_tab) is True

    # Disable cache globally.
    await bidi_session.network.set_cache_behavior(cache_behavior="bypass")

    assert await is_cache_enabled_for_context(context=top_context) is False
    assert await is_cache_enabled_for_context(context=new_tab) is False

    # Enable cache for one context.
    await bidi_session.network.set_cache_behavior(
        cache_behavior="default", contexts=[new_tab["context"]]
    )

    # Make sure that cache is disabled only for one context.
    assert await is_cache_enabled_for_context(context=top_context) is False
    assert await is_cache_enabled_for_context(context=new_tab) is True

    # Reset to default behavior.
    await bidi_session.network.set_cache_behavior(cache_behavior="default")
