import pytest

from . import ANOTHER_CLIENT_HINTS, SOME_CLIENT_HINTS

pytestmark = pytest.mark.asyncio


async def test_set_override_and_reset_globally(bidi_session,
        top_context,
        create_user_context,
        default_client_hints,
        assert_client_hints):
    user_context = await create_user_context()
    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    await bidi_session.emulation.set_client_hints_override(
        client_hints=SOME_CLIENT_HINTS
    )

    await assert_client_hints(top_context, SOME_CLIENT_HINTS)
    await assert_client_hints(context_in_user_context, SOME_CLIENT_HINTS)

    another_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab")

    await assert_client_hints(another_context_in_user_context,
                              SOME_CLIENT_HINTS)

    # Reset global override.
    await bidi_session.emulation.set_client_hints_override(
        client_hints=None
    )

    # Assert the override is reset for existing contexts.
    await assert_client_hints(top_context, default_client_hints)
    await assert_client_hints(context_in_user_context, default_client_hints)
    await assert_client_hints(another_context_in_user_context,
                              default_client_hints)
    # Assert the override is reset for new context.
    await assert_client_hints(
        await bidi_session.browsing_context.create(user_context=user_context,
                                                   type_hint="tab"),
        default_client_hints)


async def test_set_override_and_reset_globally_and_per_context(
        bidi_session, top_context, default_client_hints, assert_client_hints):
    await bidi_session.emulation.set_client_hints_override(
        contexts=[top_context["context"]],
        client_hints=SOME_CLIENT_HINTS
    )
    await assert_client_hints(top_context, SOME_CLIENT_HINTS)

    # Set global override.
    await bidi_session.emulation.set_client_hints_override(
        client_hints=ANOTHER_CLIENT_HINTS
    )

    # The override per context should still be active.
    await assert_client_hints(top_context, SOME_CLIENT_HINTS)

    # Reset per-context override.
    await bidi_session.emulation.set_client_hints_override(
        contexts=[top_context["context"]],
        client_hints=None
    )

    # The global override should be active.
    await assert_client_hints(top_context, ANOTHER_CLIENT_HINTS)

    # Reset global override.
    await bidi_session.emulation.set_client_hints_override(
        client_hints=None
    )

    # The override should be disabled.
    await assert_client_hints(top_context, default_client_hints)


async def test_set_override_and_reset_globally_and_per_user_context(
        bidi_session, top_context, default_client_hints, assert_client_hints):
    await bidi_session.emulation.set_client_hints_override(
        user_contexts=["default"],
        client_hints=SOME_CLIENT_HINTS
    )
    await assert_client_hints(top_context, SOME_CLIENT_HINTS)

    # Set global override.
    await bidi_session.emulation.set_client_hints_override(
        client_hints=ANOTHER_CLIENT_HINTS
    )

    # The override per user context should still be active.
    await assert_client_hints(top_context, SOME_CLIENT_HINTS)

    # Reset per user context override.
    await bidi_session.emulation.set_client_hints_override(
        user_contexts=["default"],
        client_hints=None
    )

    # The global override should be active.
    await assert_client_hints(top_context, ANOTHER_CLIENT_HINTS)

    # Reset global override.
    await bidi_session.emulation.set_client_hints_override(
        client_hints=None
    )

    # The override should be disabled.
    await assert_client_hints(top_context, default_client_hints)
