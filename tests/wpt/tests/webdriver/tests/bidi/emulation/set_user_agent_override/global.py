import pytest

pytestmark = pytest.mark.asyncio

SOME_USER_AGENT = "SOME_USER_AGENT"
ANOTHER_USER_AGENT = "ANOTHER_USER_AGENT"


async def test_user_agent_set_override_and_reset_globally(bidi_session,
        top_context, create_user_context, default_user_agent,
        assert_user_agent):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    await bidi_session.emulation.set_user_agent_override(
        user_agent=SOME_USER_AGENT
    )

    await assert_user_agent(top_context, SOME_USER_AGENT)
    await assert_user_agent(context_in_user_context, SOME_USER_AGENT)

    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    await assert_user_agent(another_context_in_user_context, SOME_USER_AGENT)

    # Reset global override.
    await bidi_session.emulation.set_user_agent_override(
        user_agent=None
    )

    # Assert the override is reset for existing contexts.
    await assert_user_agent(top_context, default_user_agent)
    await assert_user_agent(context_in_user_context, default_user_agent)
    await assert_user_agent(another_context_in_user_context, default_user_agent)
    # Assert the override is reset for new context.
    await assert_user_agent(
        await bidi_session.browsing_context.create(user_context=user_context,
                                                   type_hint="tab"),
        default_user_agent)


async def test_user_agent_set_override_and_reset_globally_and_per_context(
        bidi_session, top_context, default_user_agent, assert_user_agent):
    await bidi_session.emulation.set_user_agent_override(
        contexts=[top_context["context"]],
        user_agent=SOME_USER_AGENT
    )
    await assert_user_agent(top_context, SOME_USER_AGENT)

    # Set global override.
    await bidi_session.emulation.set_user_agent_override(
        user_agent=ANOTHER_USER_AGENT
    )

    # The override per context should still be active.
    await assert_user_agent(top_context, SOME_USER_AGENT)

    # Reset per-context override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[top_context["context"]],
        user_agent=None
    )

    # The global override should be active.
    await assert_user_agent(top_context, ANOTHER_USER_AGENT)

    # Reset global override.
    await bidi_session.emulation.set_user_agent_override(
        user_agent=None
    )

    # The override should be disabled.
    await assert_user_agent(top_context, default_user_agent)


async def test_user_agent_set_override_and_reset_globally_and_per_user_context(
        bidi_session, top_context, default_user_agent, assert_user_agent):
    await bidi_session.emulation.set_user_agent_override(
        user_contexts=["default"],
        user_agent=SOME_USER_AGENT
    )
    await assert_user_agent(top_context, SOME_USER_AGENT)

    # Set global override.
    await bidi_session.emulation.set_user_agent_override(
        user_agent=ANOTHER_USER_AGENT
    )

    # The override per user context should still be active.
    await assert_user_agent(top_context, SOME_USER_AGENT)

    # Reset per user context override.
    await bidi_session.emulation.set_user_agent_override(
        user_contexts=["default"],
        user_agent=None
    )

    # The global override should be active.
    await assert_user_agent(top_context, ANOTHER_USER_AGENT)

    # Reset global override.
    await bidi_session.emulation.set_user_agent_override(
        user_agent=None
    )

    # The override should be disabled.
    await assert_user_agent(top_context, default_user_agent)
