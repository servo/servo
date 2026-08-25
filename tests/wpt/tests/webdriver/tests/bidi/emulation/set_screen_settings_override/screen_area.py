import pytest


pytestmark = pytest.mark.asyncio


async def test_screen_area(
    bidi_session,
    new_tab,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    # Set screen area override.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=screen_area_override
    )

    # Assert that screen area is updated.
    await assert_screen_dimensions(
        new_tab,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Set screen area override again.
    screen_area_override_2 = {"width": 200, "height": 200}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=screen_area_override_2
    )

    # Assert that screen area is updated.
    await assert_screen_dimensions(
        new_tab,
        screen_area_override_2["width"],
        screen_area_override_2["height"],
        screen_area_override_2["width"],
        screen_area_override_2["height"],
    )

    # Reset screen area override.
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=None
    )

    # Assert screen orientation is the default.
    await assert_screen_dimensions(
        new_tab,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )
