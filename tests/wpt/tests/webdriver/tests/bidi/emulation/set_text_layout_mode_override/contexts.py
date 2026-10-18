import pytest

pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session,
    new_tab,
    top_context,
    is_text_autosizing_enabled,
    default_text_autosizing_state,
):
    # Apply textLayoutMode override to new_tab only.
    await bidi_session.emulation.set_text_layout_mode_override(
        contexts=[new_tab["context"]],
        text_layout_mode="mobile",
    )
    assert await is_text_autosizing_enabled(new_tab) is True
    assert (
        await is_text_autosizing_enabled(top_context)
        == default_text_autosizing_state
    )

    # Verify newly created context remains in default state.
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )
    assert (
        await is_text_autosizing_enabled(another_context)
        == default_text_autosizing_state
    )

    # Clear context-scoped override and verify all contexts revert to default.
    await bidi_session.emulation.set_text_layout_mode_override(
        contexts=[new_tab["context"]],
        text_layout_mode=None,
    )
    assert (
        await is_text_autosizing_enabled(new_tab)
        == default_text_autosizing_state
    )
    assert (
        await is_text_autosizing_enabled(top_context)
        == default_text_autosizing_state
    )
    assert (
        await is_text_autosizing_enabled(another_context)
        == default_text_autosizing_state
    )


async def test_overrides_user_contexts(
    bidi_session,
    affected_user_context,
    is_text_autosizing_enabled,
    default_text_autosizing_state,
):
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context,
        type_hint="tab",
    )

    # Set override at both browsing context and user context levels.
    await bidi_session.emulation.set_text_layout_mode_override(
        contexts=[context_in_user_context["context"]],
        text_layout_mode="mobile",
    )
    await bidi_session.emulation.set_text_layout_mode_override(
        user_contexts=[affected_user_context],
        text_layout_mode="mobile",
    )
    assert await is_text_autosizing_enabled(context_in_user_context) is True

    # Clear context-level setting and verify fallback to user-context setting.
    await bidi_session.emulation.set_text_layout_mode_override(
        contexts=[context_in_user_context["context"]],
        text_layout_mode=None,
    )
    assert await is_text_autosizing_enabled(context_in_user_context) is True

    # Clear user-context setting and verify fallback to default state.
    await bidi_session.emulation.set_text_layout_mode_override(
        user_contexts=[affected_user_context],
        text_layout_mode=None,
    )
    assert (
        await is_text_autosizing_enabled(context_in_user_context)
        == default_text_autosizing_state
    )


async def test_overrides_global(
    bidi_session,
    new_tab,
    is_text_autosizing_enabled,
    default_text_autosizing_state,
):
    # Set override at both browsing context and global levels.
    await bidi_session.emulation.set_text_layout_mode_override(
        contexts=[new_tab["context"]],
        text_layout_mode="mobile",
    )
    await bidi_session.emulation.set_text_layout_mode_override(
        text_layout_mode="mobile",
    )
    assert await is_text_autosizing_enabled(new_tab) is True

    # Clear context-level setting and verify fallback to active global setting.
    await bidi_session.emulation.set_text_layout_mode_override(
        contexts=[new_tab["context"]],
        text_layout_mode=None,
    )
    assert await is_text_autosizing_enabled(new_tab) is True

    # Clear global setting and verify fallback to default state.
    await bidi_session.emulation.set_text_layout_mode_override(
        text_layout_mode=None,
    )
    assert (
        await is_text_autosizing_enabled(new_tab)
        == default_text_autosizing_state
    )
