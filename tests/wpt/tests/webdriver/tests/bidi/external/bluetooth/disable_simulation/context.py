import pytest
import webdriver.bidi.error as error

from .. import TEST_DEVICE_ADDRESS, TEST_DEVICE_NAME, set_simulate_adapter


pytestmark = pytest.mark.asyncio


async def test_contexts_are_isolated(bidi_session, top_context, test_page):
    another_browsing_context = await bidi_session.browsing_context.create(
        type_hint="tab")
    await set_simulate_adapter(bidi_session, top_context, test_page,
                               "powered-on")

    await set_simulate_adapter(bidi_session, another_browsing_context,
                               test_page, "powered-on")
    await bidi_session.bluetooth.disable_simulation(context=another_browsing_context["context"])
    # Simulation commands should still work after simulation is disabled in another
    # context.
    await bidi_session.bluetooth.simulate_preconnected_peripheral(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS, name=TEST_DEVICE_NAME,
        manufacturer_data=[],
        known_service_uuids=[])
