import pytest

pytestmark = pytest.mark.asyncio


async def test_contexts(
        bidi_session, new_tab, top_context, get_screen_orientation,
        some_bidi_screen_orientation, some_web_screen_orientation,
        default_screen_orientation):
    # Set screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[new_tab["context"]],
        screen_orientation=some_bidi_screen_orientation,
    )

    # Assert screen orientation in the new context is updated.
    assert await get_screen_orientation(
        new_tab) == some_web_screen_orientation
    # Assert screen orientation in the initial context is unchanged.
    assert await get_screen_orientation(
        top_context) == default_screen_orientation

    # Reset screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[new_tab["context"]], screen_orientation=None
    )

    # Assert screen orientations are the default.
    assert await get_screen_orientation(new_tab) == default_screen_orientation
    assert await get_screen_orientation(
        top_context) == default_screen_orientation


async def test_multiple_contexts(
        bidi_session, new_tab, top_context, get_screen_orientation,
        some_bidi_screen_orientation, some_web_screen_orientation,
        default_screen_orientation):
    # Set screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[top_context["context"], new_tab["context"]],
        screen_orientation=some_bidi_screen_orientation,
    )

    # Assert screen orientations in both contexts are updated.
    assert await get_screen_orientation(new_tab) == some_web_screen_orientation
    assert await get_screen_orientation(
        top_context) == some_web_screen_orientation

    # Reset screen orientation override of the new tab.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[new_tab["context"]],
        screen_orientation=None
    )

    # Assert screen orientation on the new tab is the default.
    assert await get_screen_orientation(new_tab) == default_screen_orientation
    # Assert screen orientation on the initial tab is still updated.
    assert await get_screen_orientation(top_context) == some_web_screen_orientation

    # Reset screen orientation override of the initial tab.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[top_context["context"]],
        screen_orientation=None
    )

    # Assert screen orientations on both tabs are the default.
    assert await get_screen_orientation(new_tab) == default_screen_orientation
    assert await get_screen_orientation(
        top_context) == default_screen_orientation
