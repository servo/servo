import pytest

pytestmark = pytest.mark.asyncio


async def test_global(
    bidi_session,
    top_context,
    is_text_autosizing_enabled,
    default_text_autosizing_state,
):
    # Apply global textLayoutMode override.
    await bidi_session.emulation.set_text_layout_mode_override(
        text_layout_mode="mobile",
    )
    assert await is_text_autosizing_enabled(top_context) is True

    # Verify newly created context inherits global override.
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )
    assert await is_text_autosizing_enabled(another_context) is True

    # Clear global setting and verify all contexts revert to default state.
    await bidi_session.emulation.set_text_layout_mode_override(
        text_layout_mode=None,
    )
    assert (
        await is_text_autosizing_enabled(top_context)
        == default_text_autosizing_state
    )
    assert (
        await is_text_autosizing_enabled(another_context)
        == default_text_autosizing_state
    )
