import pytest
import webdriver.bidi.error as error

from .. import TEST_DEVICE_ADDRESS, TEST_DEVICE_NAME, set_simulate_adapter


pytestmark = pytest.mark.asyncio


async def test_disable_simulation(bidi_session, top_context, test_page):
    await set_simulate_adapter(bidi_session, top_context, test_page,
                               "powered-on")
    await bidi_session.bluetooth.disable_simulation(context=top_context["context"])
    # Creating a fake BT device while simulation disabled would fail.
    with pytest.raises(error.UnknownErrorException):
        await bidi_session.bluetooth.simulate_preconnected_peripheral(
        context=top_context["context"],
        address=TEST_DEVICE_ADDRESS, name=TEST_DEVICE_NAME,
        manufacturer_data=[],
        known_service_uuids=[])
