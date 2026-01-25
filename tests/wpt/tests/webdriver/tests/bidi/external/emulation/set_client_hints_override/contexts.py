import pytest

from . import SOME_CLIENT_HINTS

pytestmark = pytest.mark.asyncio


async def test_contexts(bidi_session, new_tab, top_context,
        default_client_hints, assert_client_hints):
    # Set client hints override.
    await bidi_session.emulation.set_client_hints_override(
        contexts=[new_tab["context"]],
        client_hints=SOME_CLIENT_HINTS
    )

    # Assert client hints override is set only in the required context.
    await assert_client_hints(new_tab, SOME_CLIENT_HINTS)
    await assert_client_hints(top_context, default_client_hints)

    # Reset client hints override.
    await bidi_session.emulation.set_client_hints_override(
        contexts=[new_tab["context"]],
        client_hints=None
    )

    # Assert client hints override is reset.
    await assert_client_hints(new_tab, default_client_hints)
    await assert_client_hints(top_context, default_client_hints)


async def test_multiple_contexts(bidi_session, new_tab, default_client_hints,
        assert_client_hints):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Set client hints override
    await bidi_session.emulation.set_client_hints_override(
        contexts=[new_tab["context"], new_context["context"]],
        client_hints=SOME_CLIENT_HINTS
    )

    # Assert client hints override is set in all the required contexts.
    await assert_client_hints(new_tab, SOME_CLIENT_HINTS)
    await assert_client_hints(new_context, SOME_CLIENT_HINTS)

    # Reset client hints override.
    await bidi_session.emulation.set_client_hints_override(
        contexts=[new_tab["context"], new_context["context"]],
        client_hints=None
    )

    # Assert client hints override is reset.
    await assert_client_hints(new_tab, default_client_hints)
    await assert_client_hints(new_context, default_client_hints)
