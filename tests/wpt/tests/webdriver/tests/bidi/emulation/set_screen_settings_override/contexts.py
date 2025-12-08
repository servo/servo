import pytest

pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session,
    new_tab,
    top_context,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    # Set screen dimensions override.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=screen_area_override
    )

    # Assert screen dimensions in the new context are updated.
    await assert_screen_dimensions(
        new_tab,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )
    # Assert screen dimensions in the initial context are unchanged.
    await assert_screen_dimensions(
        top_context,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )

    # Reset screen dimensions override.
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=None
    )

    # Assert screen dimensions in the new context are reset to default.
    await assert_screen_dimensions(
        new_tab,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )


async def test_multiple_contexts(
    bidi_session,
    new_tab,
    top_context,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    # Set screen orientation override.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[top_context["context"], new_tab["context"]],
        screen_area=screen_area_override,
    )

    # Assert screen dimensions in both contexts are updated.
    await assert_screen_dimensions(
        new_tab,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )
    await assert_screen_dimensions(
        top_context,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Reset screen dimensions override of the new tab.
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=None
    )

    # Assert screen dimensions on the new tab are default.
    await assert_screen_dimensions(
        new_tab,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )
    # Assert screen dimensions on the initial tab are still updated.
    await assert_screen_dimensions(
        top_context,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Reset screen dimensions override of the initial tab.
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[top_context["context"]], screen_area=None
    )

    # Assert screen dimensions on both tabs are default.
    await assert_screen_dimensions(
        new_tab,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )
    await assert_screen_dimensions(
        top_context,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    new_tab,
    assert_screen_dimensions,
    get_current_screen_dimensions,
    iframe,
    inline,
    domain,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    # Set screen orientation override.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]],
        screen_area=screen_area_override,
    )

    # Assert screen orientation in the required context.
    await assert_screen_dimensions(
        new_tab,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    page_url = inline(f"""{iframe("<div id='in-iframe'>foo</div>", domain=domain)}""")

    # Load the page with iframes.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=page_url,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe = contexts[0]["children"][0]

    # Assert screen dimensions are emulated in the iframe context.
    await assert_screen_dimensions(
        iframe,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Set another screen orientation override.
    screen_area_override_2 = {"width": 200, "height": 200}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]],
        screen_area=screen_area_override_2,
    )

    # Assert screen dimensions are emulated in the iframe context.
    await assert_screen_dimensions(
        iframe,
        screen_area_override_2["width"],
        screen_area_override_2["height"],
        screen_area_override_2["width"],
        screen_area_override_2["height"],
    )

    # Reset screen dimensions override.
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[new_tab["context"]], screen_area=None
    )

    # Assert screen dimensions in the iframe are default.
    await assert_screen_dimensions(
        iframe,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )
