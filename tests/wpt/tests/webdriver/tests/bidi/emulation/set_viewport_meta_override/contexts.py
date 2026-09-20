import pytest

pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session,
    new_tab,
    top_context,
    is_viewport_meta_active,
    default_viewport_meta_state,
):
    # Apply viewportMeta override to new_tab only.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[new_tab["context"]],
        viewport_meta=True,
    )
    assert await is_viewport_meta_active(new_tab) is True
    assert (
        await is_viewport_meta_active(top_context)
        == default_viewport_meta_state
    )

    # Verify newly created context remains in default state.
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )
    assert (
        await is_viewport_meta_active(another_context)
        == default_viewport_meta_state
    )

    # Clear context-scoped override and verify all contexts revert to default.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[new_tab["context"]],
        viewport_meta=None,
    )
    assert (
        await is_viewport_meta_active(new_tab) == default_viewport_meta_state
    )
    assert (
        await is_viewport_meta_active(top_context)
        == default_viewport_meta_state
    )
    assert (
        await is_viewport_meta_active(another_context)
        == default_viewport_meta_state
    )


async def test_overrides_user_contexts(
    bidi_session,
    affected_user_context,
    is_viewport_meta_active,
    default_viewport_meta_state,
):
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context,
        type_hint="tab",
    )

    # Set override at both browsing context and user context levels.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[context_in_user_context["context"]],
        viewport_meta=True,
    )
    await bidi_session.emulation.set_viewport_meta_override(
        user_contexts=[affected_user_context],
        viewport_meta=True,
    )
    assert await is_viewport_meta_active(context_in_user_context) is True

    # Clear context-level setting and verify fallback to user-context setting.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[context_in_user_context["context"]],
        viewport_meta=None,
    )
    assert await is_viewport_meta_active(context_in_user_context) is True

    # Clear user-context setting and verify fallback to default state.
    await bidi_session.emulation.set_viewport_meta_override(
        user_contexts=[affected_user_context],
        viewport_meta=None,
    )
    assert (
        await is_viewport_meta_active(context_in_user_context)
        == default_viewport_meta_state
    )


async def test_overrides_global(
    bidi_session,
    new_tab,
    is_viewport_meta_active,
    default_viewport_meta_state,
):
    # Set override at both browsing context and global levels.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[new_tab["context"]],
        viewport_meta=True,
    )
    await bidi_session.emulation.set_viewport_meta_override(
        viewport_meta=True,
    )
    assert await is_viewport_meta_active(new_tab) is True

    # Clear context-level setting and verify fallback to active global setting.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[new_tab["context"]],
        viewport_meta=None,
    )
    assert await is_viewport_meta_active(new_tab) is True

    # Clear global setting and verify fallback to default state.
    await bidi_session.emulation.set_viewport_meta_override(
        viewport_meta=None,
    )
    assert (
        await is_viewport_meta_active(new_tab) == default_viewport_meta_state
    )
