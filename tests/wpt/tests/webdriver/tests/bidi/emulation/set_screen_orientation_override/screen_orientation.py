import pytest

from . import get_angle

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("_type", ["portrait-primary", "portrait-secondary",
                                   "landscape-primary", "landscape-secondary"])
@pytest.mark.parametrize("natural", ["portrait", "landscape"])
async def test_screen_orientation(bidi_session, top_context,
        get_screen_orientation, _type, natural, default_screen_orientation):
    # Set screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[top_context["context"]],
        screen_orientation={
            "type": _type,
            "natural": natural
        })

    # Assert screen orientation is updated.
    assert (await get_screen_orientation(top_context)) == {
        "type": _type,
        "angle": get_angle(_type, natural)
    }

    # Reset screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[top_context["context"]], screen_orientation=None
    )

    # Assert screen orientation is the default.
    assert await get_screen_orientation(
        top_context) == default_screen_orientation
