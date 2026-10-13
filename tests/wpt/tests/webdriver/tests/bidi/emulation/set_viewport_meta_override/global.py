import pytest

pytestmark = pytest.mark.asyncio


async def test_global(
    bidi_session,
    top_context,
    is_viewport_meta_active,
    default_viewport_meta_state,
):
    # Apply global viewportMeta override.
    await bidi_session.emulation.set_viewport_meta_override(
        viewport_meta=True,
    )
    assert await is_viewport_meta_active(top_context) is True

    # Verify newly created context inherits global override.
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )
    assert await is_viewport_meta_active(another_context) is True

    # Clear global setting and verify all contexts revert to default state.
    await bidi_session.emulation.set_viewport_meta_override(
        viewport_meta=None,
    )
    assert (
        await is_viewport_meta_active(top_context)
        == default_viewport_meta_state
    )
    assert (
        await is_viewport_meta_active(another_context)
        == default_viewport_meta_state
    )
