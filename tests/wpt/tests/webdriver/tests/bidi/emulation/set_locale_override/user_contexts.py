import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
        bidi_session,
        create_user_context,
        new_tab,
        get_current_locale,
        default_locale,
        some_locale
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    assert await get_current_locale(new_tab) == default_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context],
        locale=some_locale)

    # Assert the locale is emulated in user context.
    assert await get_current_locale(context_in_user_context) == some_locale

    # Assert the default user context is not affected.
    assert await get_current_locale(new_tab) == default_locale

    # Create a new context in the user context.
    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")
    # Assert the locale is emulated in a new browsing context of the user context.
    assert await get_current_locale(
        another_context_in_user_context) == some_locale


async def test_set_to_default_user_context(
        bidi_session,
        new_tab,
        create_user_context,
        get_current_locale,
        default_locale,
        some_locale
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    await bidi_session.emulation.set_locale_override(
        user_contexts=["default"],
        locale=some_locale,
    )

    # Make sure that the locale changes are only applied to the context
    # associated with default user context.
    assert await get_current_locale(context_in_user_context) == default_locale
    assert await get_current_locale(new_tab) == some_locale

    # Create a new context in the default context.
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    assert await get_current_locale(context_in_default_context) == some_locale
    assert await get_current_locale(context_in_default_context) == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        user_contexts=["default"],
        locale=None
    )


async def test_set_to_multiple_user_contexts(
        bidi_session,
        create_user_context,
        get_current_locale,
        some_locale,
):
    user_context_1 = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    user_context_2 = await create_user_context()
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context_1, user_context_2],
        locale=some_locale
    )

    assert await get_current_locale(context_in_user_context_1) == some_locale
    assert await get_current_locale(context_in_user_context_2) == some_locale


async def test_set_to_user_context_and_then_to_context(
        bidi_session,
        create_user_context,
        new_tab,
        get_current_locale,
        default_locale,
        some_locale,
        another_locale
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply locale override to the user context.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context],
        locale=some_locale
    )

    # Apply locale override now only to the context.
    await bidi_session.emulation.set_locale_override(
        contexts=[context_in_user_context_1["context"]],
        locale=another_locale
    )
    assert await get_current_locale(context_in_user_context_1) == another_locale

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the locale is still updated.
    assert await get_current_locale(context_in_user_context_1) == another_locale

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the locale override for the user context is applied.
    assert await get_current_locale(context_in_user_context_2) == some_locale

    await bidi_session.emulation.set_locale_override(
        contexts=[context_in_user_context_1["context"]],
        locale=None,
    )

    # Make sure that the locale override was reset.
    assert await get_current_locale(context_in_user_context_1) == default_locale
