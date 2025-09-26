import pytest

pytestmark = pytest.mark.asyncio

SOME_USER_AGENT = "SOME_USER_AGENT"
ANOTHER_USER_AGENT = "ANOTHER_USER_AGENT"


async def test_user_contexts(bidi_session, create_user_context, new_tab,
        assert_user_agent, default_user_agent):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    await assert_user_agent(new_tab, default_user_agent)

    # Set user-agent override
    await bidi_session.emulation.set_user_agent_override(
        user_contexts=[user_context],
        user_agent=SOME_USER_AGENT)

    # Assert user-agent override is set in user context.
    await assert_user_agent(context_in_user_context, SOME_USER_AGENT)

    # Assert the default user context is not affected.
    await assert_user_agent(new_tab, default_user_agent)

    # Create a new context in the user context.
    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")
    # Assert user-agent override is set in a new browsing context of the user context.
    await assert_user_agent(
        another_context_in_user_context, SOME_USER_AGENT)


async def test_set_to_default_user_context(bidi_session, new_tab,
        create_user_context, assert_user_agent, default_user_agent, ):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    await bidi_session.emulation.set_user_agent_override(
        user_contexts=["default"],
        user_agent=SOME_USER_AGENT,
    )

    # Make sure that the user-agent override applied only to the context
    # associated with default user context.
    await assert_user_agent(context_in_user_context, default_user_agent)
    await assert_user_agent(new_tab, SOME_USER_AGENT)

    # Create a new context in the default context.
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    await assert_user_agent(context_in_default_context, SOME_USER_AGENT)

    # Reset user-agent override.
    await bidi_session.emulation.set_user_agent_override(
        user_contexts=["default"],
        user_agent=None
    )


async def test_set_to_multiple_user_contexts(bidi_session, create_user_context,
        assert_user_agent, default_user_agent):
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
    # Set user-agent override for both user contexts.
    await bidi_session.emulation.set_user_agent_override(
        user_contexts=[user_context_1, user_context_2],
        user_agent=SOME_USER_AGENT
    )

    # Assert user-agent override is set in the browsing context of the first
    # user context.
    await assert_user_agent(context_in_user_context_1, SOME_USER_AGENT)
    # Assert user-agent override is set in the browsing context of the second
    # user context.
    await assert_user_agent(context_in_user_context_2, SOME_USER_AGENT)


async def test_set_to_user_context_and_then_to_context(bidi_session,
        create_user_context, new_tab, assert_user_agent, default_user_agent, ):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Add user context override.
    await bidi_session.emulation.set_user_agent_override(
        user_contexts=[user_context],
        user_agent=SOME_USER_AGENT
    )

    # Add context override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[context_in_user_context["context"]],
        user_agent=ANOTHER_USER_AGENT
    )
    # Assert context override is applied.
    await assert_user_agent(context_in_user_context, ANOTHER_USER_AGENT)

    # Reload and assert context override is still applied.
    await bidi_session.browsing_context.reload(
        context=context_in_user_context["context"], wait="complete"
    )
    await assert_user_agent(context_in_user_context, ANOTHER_USER_AGENT)

    # Remove context override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[context_in_user_context["context"]],
        user_agent=None
    )

    # Assert user context override is applied.
    await assert_user_agent(context_in_user_context, SOME_USER_AGENT)

    # Reload and assert user context override is still applied.
    await bidi_session.browsing_context.reload(
        context=context_in_user_context["context"], wait="complete"
    )
    await assert_user_agent(context_in_user_context, SOME_USER_AGENT)

    # Remove user context override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[context_in_user_context["context"]],
        user_agent=None,
    )

    # Assert the overrides are removed.
    await assert_user_agent(context_in_user_context, default_user_agent)

    # Reload and assert the overrides are still removed.
    await bidi_session.browsing_context.reload(
        context=context_in_user_context["context"], wait="complete"
    )
    await assert_user_agent(context_in_user_context, SOME_USER_AGENT)
