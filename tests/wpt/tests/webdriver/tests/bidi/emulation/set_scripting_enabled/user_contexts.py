import pytest

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
        bidi_session,
        create_user_context,
        new_tab,
        is_scripting_enabled,
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    assert await is_scripting_enabled(new_tab) is True

    # Disable scripting
    await bidi_session.emulation.set_scripting_enabled(
        user_contexts=[user_context],
        enabled=False)

    # Assert scripting is disabled in user context.
    assert await is_scripting_enabled(context_in_user_context) is False

    # Assert the default user context is not affected.
    assert await is_scripting_enabled(new_tab) is True

    # Create a new context in the user context.
    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")
    # Assert scripting is disabled in a new browsing context of the user context.
    assert await is_scripting_enabled(
        another_context_in_user_context) is False


async def test_set_to_default_user_context(
        bidi_session,
        new_tab,
        create_user_context,
        is_scripting_enabled,
):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    await bidi_session.emulation.set_scripting_enabled(
        user_contexts=["default"],
        enabled=False,
    )

    # Make sure that the scripting changes are only applied to the context
    # associated with default user context.
    assert await is_scripting_enabled(context_in_user_context) is True
    assert await is_scripting_enabled(new_tab) is False

    # Create a new context in the default context.
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    assert await is_scripting_enabled(context_in_default_context) is False

    # Reset scripting override.
    await bidi_session.emulation.set_scripting_enabled(
        user_contexts=["default"],
        enabled=None
    )


async def test_set_to_multiple_user_contexts(
        bidi_session,
        create_user_context,
        is_scripting_enabled,
):
    # Create the first user context.
    user_context_1 = await create_user_context()
    # Create a browsing context within the first user context.
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    # Create the second user context.
    user_context_2 = await create_user_context()
    # Create a browsing context within the second user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )
    # Disable scripting for both user contexts.
    await bidi_session.emulation.set_scripting_enabled(
        user_contexts=[user_context_1, user_context_2],
        enabled=False
    )

    # Assert scripting is disabled in the browsing context of the first user context.
    assert await is_scripting_enabled(context_in_user_context_1) is False
    # Assert scripting is disabled in the browsing context of the second user context.
    assert await is_scripting_enabled(context_in_user_context_2) is False


async def test_set_to_user_context_and_then_to_context(
        bidi_session,
        create_user_context,
        new_tab,
        is_scripting_enabled,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Apply scripting override to the user context.
    await bidi_session.emulation.set_scripting_enabled(
        user_contexts=[user_context],
        enabled=False
    )

    # Apply scripting override now only to the context.
    await bidi_session.emulation.set_scripting_enabled(
        contexts=[context_in_user_context_1["context"]],
        enabled=None
    )
    assert await is_scripting_enabled(context_in_user_context_1) is True

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the scripting is still updated.
    assert await is_scripting_enabled(context_in_user_context_1) is True

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the scripting override for the user context is applied.
    assert await is_scripting_enabled(context_in_user_context_2) is False

    await bidi_session.emulation.set_scripting_enabled(
        contexts=[context_in_user_context_1["context"]],
        enabled=None,
    )

    # Make sure that the scripting override was reset.
    assert await is_scripting_enabled(context_in_user_context_1) is True
