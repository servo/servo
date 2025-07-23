import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
        bidi_session,
        create_user_context,
        new_tab,
        get_current_timezone,
        default_timezone,
        some_timezone
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    assert await get_current_timezone(new_tab) == default_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        user_contexts=[user_context],
        timezone=some_timezone)

    # Assert the timezone is emulated in user context.
    assert await get_current_timezone(context_in_user_context) == some_timezone

    # Assert the default user context is not affected.
    assert await get_current_timezone(new_tab) == default_timezone

    # Create a new context in the user context.
    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")
    # Assert the timezone is emulated in a new browsing context of the user context.
    assert await get_current_timezone(
        another_context_in_user_context) == some_timezone


async def test_set_to_default_user_context(
        bidi_session,
        new_tab,
        create_user_context,
        get_current_timezone,
        default_timezone,
        some_timezone
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    await bidi_session.emulation.set_timezone_override(
        user_contexts=["default"],
        timezone=some_timezone,
    )

    # Make sure that the timezone changes are only applied to the context
    # associated with default user context.
    assert await get_current_timezone(
        context_in_user_context) == default_timezone
    assert await get_current_timezone(new_tab) == some_timezone

    # Create a new context in the default context.
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    assert await get_current_timezone(
        context_in_default_context) == some_timezone
    assert await get_current_timezone(
        context_in_default_context) == some_timezone

    # Reset timezone override.
    await bidi_session.emulation.set_timezone_override(
        user_contexts=["default"],
        timezone=None
    )


async def test_set_to_multiple_user_contexts(
        bidi_session,
        create_user_context,
        get_current_timezone,
        some_timezone,
):
    user_context_1 = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    user_context_2 = await create_user_context()
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )
    await bidi_session.emulation.set_timezone_override(
        user_contexts=[user_context_1, user_context_2],
        timezone=some_timezone
    )

    assert await get_current_timezone(
        context_in_user_context_1) == some_timezone
    assert await get_current_timezone(
        context_in_user_context_2) == some_timezone


async def test_set_to_user_context_and_then_to_context(
        bidi_session,
        create_user_context,
        new_tab,
        get_current_timezone,
        default_timezone,
        some_timezone,
        another_timezone
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply timezone override to the user context.
    await bidi_session.emulation.set_timezone_override(
        user_contexts=[user_context],
        timezone=some_timezone
    )

    # Apply timezone override now only to the context.
    await bidi_session.emulation.set_timezone_override(
        contexts=[context_in_user_context_1["context"]],
        timezone=another_timezone
    )
    assert await get_current_timezone(
        context_in_user_context_1) == another_timezone

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the timezone is still updated.
    assert await get_current_timezone(
        context_in_user_context_1) == another_timezone

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the timezone override for the user context is applied.
    assert await get_current_timezone(
        context_in_user_context_2) == some_timezone

    await bidi_session.emulation.set_timezone_override(
        contexts=[context_in_user_context_1["context"]],
        timezone=None,
    )

    # Make sure that the timezone override was reset.
    assert await get_current_timezone(
        context_in_user_context_1) == default_timezone
