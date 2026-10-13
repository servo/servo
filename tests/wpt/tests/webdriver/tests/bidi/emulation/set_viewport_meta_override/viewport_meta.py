import pytest

pytestmark = pytest.mark.asyncio


async def test_viewport_meta(
    bidi_session,
    new_tab,
    is_viewport_meta_active,
    default_viewport_meta_state,
):
    # Note: The user agent's default viewport meta state can be either enabled
    # or disabled depending on the platform/configuration. This test verifies
    # that setting `viewport_meta=True` force-enables viewport meta tag parsing.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[new_tab["context"]],
        viewport_meta=True,
    )
    assert await is_viewport_meta_active(new_tab) is True

    # Clear viewport meta tag override and verify fallback to default state.
    await bidi_session.emulation.set_viewport_meta_override(
        contexts=[new_tab["context"]],
        viewport_meta=None,
    )
    assert (
        await is_viewport_meta_active(new_tab) == default_viewport_meta_state
    )
