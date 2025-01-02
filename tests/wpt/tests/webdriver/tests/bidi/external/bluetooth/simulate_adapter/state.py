import pytest

from . import get_bluetooth_availability, set_simulate_adapter

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("state,availability",
                         [("absent", False), ("powered-off", True),
                          ("powered-on", True)])
async def test_state(bidi_session, top_context, test_page, state, availability):
    await set_simulate_adapter(bidi_session, top_context, test_page, state)
    assert await get_bluetooth_availability(bidi_session,
                                            top_context) == availability


@pytest.mark.parametrize("state_1,availability_1",
                         [("absent", False), ("powered-off", True),
                          ("powered-on", True)])
@pytest.mark.parametrize("state_2,availability_2",
                         [("absent", False), ("powered-off", True),
                          ("powered-on", True)])
async def test_set_twice(bidi_session, top_context, test_page, state_1,
      availability_1, state_2, availability_2):
    await set_simulate_adapter(bidi_session, top_context, test_page, state_1)
    assert await get_bluetooth_availability(bidi_session,
                                            top_context) == availability_1

    await set_simulate_adapter(bidi_session, top_context, test_page, state_2)
    assert await get_bluetooth_availability(bidi_session,
                                            top_context) == availability_2
