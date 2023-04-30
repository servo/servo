import pytest

from webdriver.bidi.modules.input import Actions
from .. import get_events

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("delta_x, delta_y", [(0, 10), (5, 0), (5, 10)])
async def test_wheel_scroll(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    actions = Actions()

    outer = await get_element("#outer")
    actions.add_wheel().scroll(x=0, y=0, delta_x=delta_x, delta_y=delta_y, origin=outer)

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    events = await get_events(bidi_session, top_context["context"])

    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] >= delta_x
    assert events[0]["deltaY"] >= delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "outer"


@pytest.mark.parametrize("delta_x, delta_y", [(0, 10), (5, 0), (5, 10)])
async def test_wheel_scroll_iframe(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    actions = Actions()

    subframe = await get_element("#subframe")
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_x, delta_y=delta_y, origin=subframe
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] >= delta_x
    assert events[0]["deltaY"] >= delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"


@pytest.mark.parametrize("delta_x, delta_y", [(0, 10), (5, 0), (5, 10)])
async def test_wheel_scroll_overflow(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    actions = Actions()

    scrollable = await get_element("#scrollable")

    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_x, delta_y=delta_y, origin=scrollable
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] >= delta_x
    assert events[0]["deltaY"] >= delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollContent"
