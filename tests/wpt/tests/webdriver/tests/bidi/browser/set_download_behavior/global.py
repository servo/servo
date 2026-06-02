import pytest

pytestmark = pytest.mark.asyncio


async def test_set_and_reset_globally(bidi_session, temp_dir, top_context,
        is_download_allowed, is_download_allowed_invariant, trigger_download,
        create_user_context, some_download_behavior,
        opposite_download_behavior):
    default_download_allowed = await is_download_allowed(
        top_context)

    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")
    default_user_context_download_allowed = await is_download_allowed(
        context_in_user_context)

    # Set custom download behavior globally.
    await bidi_session.browser.set_download_behavior(
        download_behavior=some_download_behavior)

    assert await is_download_allowed(
        top_context) == is_download_allowed_invariant
    assert await is_download_allowed(
        await bidi_session.browsing_context.create(user_context="default",
                                                   type_hint="tab")) == is_download_allowed_invariant

    assert await is_download_allowed(
        context_in_user_context) == is_download_allowed_invariant
    assert await is_download_allowed(
        await bidi_session.browsing_context.create(user_context=user_context,
                                                   type_hint="tab")) == is_download_allowed_invariant

    assert await is_download_allowed(
        context_in_user_context) == is_download_allowed_invariant

    # Reset global behavior.
    await bidi_session.browser.set_download_behavior(download_behavior=None)

    # Assert the override is reset for default user context.
    assert await is_download_allowed(top_context) == default_download_allowed
    assert await is_download_allowed(
        await bidi_session.browsing_context.create(user_context="default",
                                                   type_hint="tab")) == default_download_allowed

    # Assert the override is reset for custom user context.
    assert await is_download_allowed(
        context_in_user_context) == default_user_context_download_allowed
    assert await is_download_allowed(
        await bidi_session.browsing_context.create(user_context=user_context,
                                                   type_hint="tab")) == default_user_context_download_allowed


async def test_set_and_reset_globally_and_per_user_context(bidi_session,
        temp_dir, is_download_allowed, is_download_allowed_invariant,
        trigger_download, create_user_context, some_download_behavior,
        opposite_download_behavior, user_context_invariant):
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context_invariant, type_hint="tab")

    default_download_allowed = await is_download_allowed(
        context_in_user_context)

    # Set behavior per user context.
    await bidi_session.browser.set_download_behavior(
        download_behavior=some_download_behavior,
        user_contexts=[user_context_invariant])

    # The behavior should be set.
    assert await is_download_allowed(
        context_in_user_context) == is_download_allowed_invariant

    # Set global override.
    await bidi_session.browser.set_download_behavior(
        download_behavior=opposite_download_behavior)

    # The behavior per user context should still be active.
    assert await is_download_allowed(
        context_in_user_context) == is_download_allowed_invariant

    # Reset per user context override.
    await bidi_session.browser.set_download_behavior(download_behavior=None,
                                                     user_contexts=[
                                                         user_context_invariant])

    # The global override should be active. `opposite_download_behavior` is
    # opposite to `download_behaviors_invariant`.
    assert await is_download_allowed(context_in_user_context) == (
        not is_download_allowed_invariant)

    # Reset global behavior.
    await bidi_session.browser.set_download_behavior(download_behavior=None)

    # The override should be disabled.
    assert await is_download_allowed(
        context_in_user_context) == default_download_allowed
