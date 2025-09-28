import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(bidi_session, temp_dir, is_download_allowed,
        is_download_allowed_invariant, trigger_download, create_user_context,
        some_download_behavior, user_context_invariant):
    affected_user_context = await create_user_context() if user_context_invariant == "new" else "default"
    not_affected_user_context = "default" if user_context_invariant == "new" else await create_user_context()

    context_in_affected_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context, type_hint="tab")
    context_in_not_affected_user_context = await bidi_session.browsing_context.create(
        user_context=not_affected_user_context, type_hint="tab")
    # Default can be different from different user contexts.
    default_affected_context_download_allowed = await is_download_allowed(
        context_in_affected_user_context)
    default_not_affected_context_download_allowed = await is_download_allowed(
        context_in_not_affected_user_context)

    # Set custom download behavior.
    await bidi_session.browser.set_download_behavior(
        download_behavior=some_download_behavior,
        user_contexts=[affected_user_context])

    # Assert behavior is set in affected user context.
    assert await is_download_allowed(
        context_in_affected_user_context) == is_download_allowed_invariant

    # Create a new context in the affected user context.
    another_context_in_affected_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context, type_hint="tab")
    # Assert behavior is set in a new browsing context of the affected user context.
    assert await is_download_allowed(
        another_context_in_affected_user_context) == is_download_allowed_invariant

    # Assert behavior is not set in not affected user context.
    assert await is_download_allowed(
        context_in_not_affected_user_context) == default_not_affected_context_download_allowed

    # Create a new context in not affected user context.
    another_context_in_not_affected_user_context = await bidi_session.browsing_context.create(
        user_context=not_affected_user_context, type_hint="tab")
    # Assert behavior is not set in a new browsing context of not affected user context.
    assert await is_download_allowed(
        another_context_in_not_affected_user_context) == default_not_affected_context_download_allowed

    # Reset to default behavior.
    await bidi_session.browser.set_download_behavior(
        download_behavior=None,
        user_contexts=[affected_user_context])

    # Assert behavior is reset in affected user context.
    assert await is_download_allowed(
        context_in_affected_user_context) == default_affected_context_download_allowed

    # Create a new context in affected user context.
    another_context_in_affected_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context, type_hint="tab")
    # Assert behavior is reset in a new browsing context of affected user context.
    assert await is_download_allowed(
        another_context_in_affected_user_context) == default_affected_context_download_allowed

    # Assert behavior is reset in not affected user context.
    assert await is_download_allowed(
        context_in_not_affected_user_context) == default_not_affected_context_download_allowed

    # Create a new context in not affected user context.
    another_context_in_not_affected_user_context = await bidi_session.browsing_context.create(
        user_context=not_affected_user_context, type_hint="tab")
    # Assert behavior is reset in a new browsing context of not affected user context.
    assert await is_download_allowed(
        another_context_in_not_affected_user_context) == default_not_affected_context_download_allowed
