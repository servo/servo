import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
    bidi_session,
    create_user_context,
    new_tab,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    await assert_locale_against_default(new_tab)

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context], locale=some_locale
    )

    # Assert the locale is emulated in user context.
    await assert_locale_against_value(some_locale, context_in_user_context)

    # Assert the default user context is not affected.
    await assert_locale_against_default(new_tab)

    # Create a new context in the user context.
    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Assert the locale is emulated in a new browsing context of the user context.
    await assert_locale_against_value(some_locale, another_context_in_user_context)


async def test_set_to_default_user_context(
    bidi_session,
    new_tab,
    create_user_context,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
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
    await assert_locale_against_default(context_in_user_context)
    await assert_locale_against_value(some_locale, new_tab)

    # Create a new context in the default context.
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    await assert_locale_against_value(some_locale, context_in_default_context)

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        user_contexts=["default"], locale=None
    )


async def test_set_to_multiple_user_contexts(
    bidi_session,
    create_user_context,
    assert_locale_against_value,
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
        user_contexts=[user_context_1, user_context_2], locale=some_locale
    )

    await assert_locale_against_value(some_locale, context_in_user_context_1)
    await assert_locale_against_value(some_locale, context_in_user_context_2)


async def test_set_to_user_context_and_then_to_context(
    bidi_session,
    create_user_context,
    another_locale,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply locale override to the user context.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context], locale=some_locale
    )

    # Apply locale override now only to the context.
    await bidi_session.emulation.set_locale_override(
        contexts=[context_in_user_context_1["context"]], locale=another_locale
    )
    await assert_locale_against_value(another_locale, context_in_user_context_1)

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the locale is still updated.
    await assert_locale_against_value(another_locale, context_in_user_context_1)

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the locale override for the user context is applied.
    await assert_locale_against_value(some_locale, context_in_user_context_2)

    # Reset the override for context.
    await bidi_session.emulation.set_locale_override(
        contexts=[context_in_user_context_1["context"]],
        locale=None,
    )

    # Make sure that the locale override is set to user context value.
    await assert_locale_against_value(some_locale, context_in_user_context_1)

    # Reset the override for user context.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context],
        locale=None,
    )

    # Make sure that the locale override is reset.
    await assert_locale_against_default(context_in_user_context_1)


async def test_set_to_context_and_then_to_user_context(
    bidi_session,
    create_user_context,
    another_locale,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply locale override to the context.
    await bidi_session.emulation.set_locale_override(
        contexts=[context_in_user_context_1["context"]], locale=some_locale
    )

    await assert_locale_against_value(some_locale, context_in_user_context_1)

    # Apply locale override to the user context.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context], locale=another_locale
    )

    # Make sure that context has still the context locale override.
    await assert_locale_against_value(some_locale, context_in_user_context_1)

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the locale still has the context locale override.
    await assert_locale_against_value(some_locale, context_in_user_context_1)

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the locale override for the user context is applied.
    await assert_locale_against_value(another_locale, context_in_user_context_2)

    # Reset override for user context.
    await bidi_session.emulation.set_locale_override(
        user_contexts=[user_context],
        locale=None,
    )

    # Make sure that the locale override for the first context is still set.
    await assert_locale_against_value(some_locale, context_in_user_context_1)
    # Make sure that the locale override for the second context is reset.
    await assert_locale_against_default(context_in_user_context_2)
