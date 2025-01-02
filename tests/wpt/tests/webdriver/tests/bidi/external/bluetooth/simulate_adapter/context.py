import pytest

from . import get_bluetooth_availability, set_simulate_adapter

pytestmark = pytest.mark.asyncio


async def test_contexts_are_isolated(bidi_session, top_context, test_page):
    another_browsing_context = await bidi_session.browsing_context.create(
        type_hint="tab")

    await set_simulate_adapter(bidi_session, top_context, test_page,
                               "powered-on")
    await set_simulate_adapter(bidi_session, another_browsing_context,
                               test_page, "absent")

    assert await get_bluetooth_availability(bidi_session, top_context) is True
    assert await get_bluetooth_availability(bidi_session,
                                            another_browsing_context) is False
