import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
    bidi_session,
    create_user_context,
    new_tab,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Set screen dimensions override.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context], screen_area=screen_area_override
    )

    # Assert screen dimensions in the new context are updated.
    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )
    # Assert screen dimensions in the initial context are unchanged.
    await assert_screen_dimensions(
        new_tab,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Assert screen dimensions in the new context are updated.
    await assert_screen_dimensions(
        context_in_user_context_2,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )


async def test_set_to_default_user_context(
    bidi_session,
    new_tab,
    create_user_context,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Set screen dimensions override.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=["default"], screen_area=screen_area_override
    )

    # Make sure that the screen dimensions changes are only applied to the
    # context associated with default user context.
    await assert_screen_dimensions(
        new_tab,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )
    # Assert screen dimensions in the initial context are unchanged.
    await assert_screen_dimensions(
        context_in_user_context_1,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )

    # Create a new context in the default context.
    context_in_default_context_2 = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    await assert_screen_dimensions(
        context_in_default_context_2,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Reset screen dimensions override.
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=["default"], screen_area=None
    )


async def test_set_to_multiple_user_contexts(
    bidi_session,
    new_tab,
    create_user_context,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    user_context_1 = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    user_context_2 = await create_user_context()
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )

    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context_1, user_context_2], screen_area=screen_area_override
    )

    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )
    await assert_screen_dimensions(
        context_in_user_context_2,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Reset screen dimensions override for one of the user contexts.
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context_1], screen_area=None
    )

    # Assert the screen dimensions override was reset for the proper user
    # context.
    await assert_screen_dimensions(
        context_in_user_context_1,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )
    await assert_screen_dimensions(
        context_in_user_context_2,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )


async def test_set_to_user_context_and_then_to_context(
    bidi_session,
    create_user_context,
    new_tab,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply screen settings override to the user context.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context], screen_area=screen_area_override
    )

    # Apply screen settings override now only to the context.
    another_screen_area_override = {"width": 200, "height": 200}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[context_in_user_context_1["context"]],
        screen_area=another_screen_area_override,
    )
    await assert_screen_dimensions(
        context_in_user_context_1,
        another_screen_area_override["width"],
        another_screen_area_override["height"],
        another_screen_area_override["width"],
        another_screen_area_override["height"],
    )

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the screen dimensions are still updated.
    await assert_screen_dimensions(
        context_in_user_context_1,
        another_screen_area_override["width"],
        another_screen_area_override["height"],
        another_screen_area_override["width"],
        another_screen_area_override["height"],
    )

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the screen settings override for the user context is
    # applied.
    await assert_screen_dimensions(
        context_in_user_context_2,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    await bidi_session.emulation.set_screen_settings_override(
        contexts=[context_in_user_context_1["context"]], screen_area=None
    )

    # Make sure that the user context screen dimensions override is applied.
    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Reset override for user context.
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context], screen_area=None
    )

    # Make sure that the override is reset.
    await assert_screen_dimensions(
        context_in_user_context_2,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )


async def test_set_to_context_and_then_to_user_context(
    bidi_session,
    create_user_context,
    new_tab,
    assert_screen_dimensions,
    get_current_screen_dimensions,
):
    default_screen_dimensions = await get_current_screen_dimensions(new_tab)

    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply screen settings override to the context.
    screen_area_override = {"width": 100, "height": 100}
    await bidi_session.emulation.set_screen_settings_override(
        contexts=[context_in_user_context_1["context"]],
        screen_area=screen_area_override,
    )

    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Apply screen settings override to the user context.
    another_screen_area_override = {"width": 200, "height": 200}
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context],
        screen_area=another_screen_area_override,
    )

    # Make sure that context has still the context screen settings override.
    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the screen dimensions still has the context screen dimensions override.
    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the screen dimensions override for the user context is applied.
    await assert_screen_dimensions(
        context_in_user_context_2,
        another_screen_area_override["width"],
        another_screen_area_override["height"],
        another_screen_area_override["width"],
        another_screen_area_override["height"],
    )

    # Reset override for user context.
    await bidi_session.emulation.set_screen_settings_override(
        user_contexts=[user_context],
        screen_area=None,
    )

    # Make sure that the screen dimensions override for the first context is still set.
    await assert_screen_dimensions(
        context_in_user_context_1,
        screen_area_override["width"],
        screen_area_override["height"],
        screen_area_override["width"],
        screen_area_override["height"],
    )
    # Make sure that the screen dimensions override for the second context is reset.
    await assert_screen_dimensions(
        context_in_user_context_2,
        default_screen_dimensions["width"],
        default_screen_dimensions["height"],
        default_screen_dimensions["availWidth"],
        default_screen_dimensions["availHeight"],
    )
