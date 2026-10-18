import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
    bidi_session,
    affected_user_context,
    not_affected_user_context,
    is_text_autosizing_enabled,
    default_text_autosizing_state,
):
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context,
        type_hint="tab",
    )
    context_in_other_user_context = await bidi_session.browsing_context.create(
        user_context=not_affected_user_context,
        type_hint="tab",
    )

    # Apply textLayoutMode override to affected_user_context.
    await bidi_session.emulation.set_text_layout_mode_override(
        user_contexts=[affected_user_context],
        text_layout_mode="mobile",
    )
    assert await is_text_autosizing_enabled(context_in_user_context) is True
    assert (
        await is_text_autosizing_enabled(context_in_other_user_context)
        == default_text_autosizing_state
    )

    # Verify newly created context in affected_user_context inherits override.
    another_context_in_user_context = (
        await bidi_session.browsing_context.create(
            user_context=affected_user_context,
            type_hint="tab",
        )
    )
    assert (
        await is_text_autosizing_enabled(another_context_in_user_context)
        is True
    )

    # Verify newly created context in not_affected_user_context stays default.
    another_context_in_other_user_context = (
        await bidi_session.browsing_context.create(
            user_context=not_affected_user_context,
            type_hint="tab",
        )
    )
    assert (
        await is_text_autosizing_enabled(another_context_in_other_user_context)
        == default_text_autosizing_state
    )

    # Clear user context setting and verify all contexts revert to default.
    await bidi_session.emulation.set_text_layout_mode_override(
        user_contexts=[affected_user_context],
        text_layout_mode=None,
    )
    assert (
        await is_text_autosizing_enabled(context_in_user_context)
        == default_text_autosizing_state
    )
    assert (
        await is_text_autosizing_enabled(another_context_in_user_context)
        == default_text_autosizing_state
    )
    assert (
        await is_text_autosizing_enabled(context_in_other_user_context)
        == default_text_autosizing_state
    )
    assert (
        await is_text_autosizing_enabled(another_context_in_other_user_context)
        == default_text_autosizing_state
    )


async def test_overrides_global(
    bidi_session,
    affected_user_context,
    is_text_autosizing_enabled,
    default_text_autosizing_state,
):
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context,
        type_hint="tab",
    )

    # Set override at both user context and global levels.
    await bidi_session.emulation.set_text_layout_mode_override(
        user_contexts=[affected_user_context],
        text_layout_mode="mobile",
    )
    await bidi_session.emulation.set_text_layout_mode_override(
        text_layout_mode="mobile",
    )
    assert await is_text_autosizing_enabled(context_in_user_context) is True

    # Clear user-context setting and verify fallback to active global setting.
    await bidi_session.emulation.set_text_layout_mode_override(
        user_contexts=[affected_user_context],
        text_layout_mode=None,
    )
    assert await is_text_autosizing_enabled(context_in_user_context) is True

    # Clear global setting and verify fallback to default state.
    await bidi_session.emulation.set_text_layout_mode_override(
        text_layout_mode=None,
    )
    assert (
        await is_text_autosizing_enabled(context_in_user_context)
        == default_text_autosizing_state
    )
