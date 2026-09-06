import pytest
import pytest_asyncio

from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from .. import wait_for_events

pytestmark = pytest.mark.asyncio


@pytest_asyncio.fixture
async def setup_event_recorder(bidi_session, top_context, inline):
    async def _setup_event_recorder(event_type):
        # "touch-action: none" keeps the synthesized touch pointers from being
        # consumed as a scroll gesture.
        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=inline(
                "<style>"
                "  html, body { margin: 0; touch-action: none; }"
                "  div { width: 200px; height: 200px; display: inline-block; }"
                "</style>"
                "<div id='target0'></div><div id='target1'></div>"
            ),
            wait="complete",
        )

        await bidi_session.script.call_function(
            function_declaration="""(eventType) => {
                window.allEvents = { events: [] };
                window.addEventListener(
                    eventType,
                    event => window.allEvents.events.push({
                        target: event.target.id,
                        changedTouches: event.changedTouches.length,
                        touches: event.touches.length,
                    })
                );
            }""",
            arguments=[{"type": "string", "value": event_type}],
            await_promise=False,
            target=ContextTarget(top_context["context"]),
        )

    return _setup_event_recorder


async def test_two_pointer_ups_in_one_tick_are_simultaneous(
    bidi_session, top_context, setup_event_recorder, get_element
):
    """Two touch pointers released in the same tick must be released at the same moment.

    The Actions API "divide[s] time into a series of ticks", and the remote end
    "will dispatch the first action of each source together, then the second
    actions together, and lastly, the final actions together"
    (https://w3c.github.io/webdriver/#actions, example 11).

    Touch Events makes that observable without measuring any timing, because
    changedTouches is defined per moment rather than per event target: for
    touchend it is "a list of the touch points that have just been removed from
    the surface".  Two pointers released from different elements within one tick
    are removed at the same moment, so both touchend events must report both
    changed touch points.  A remote end that dispatches the two pointerUp
    actions as separate device events produces the same two touchend events, but
    each reporting only a single changed touch point.

    The events are compared target-sorted because the order in which a user
    agent dispatches the events for the several targets of one moment is not
    defined.
    """
    await setup_event_recorder("touchend")

    target0 = await get_element("#target0")
    target1 = await get_element("#target1")

    actions = Actions()
    (
        actions.add_pointer(input_id="finger1", pointer_type="touch")
        .pointer_move(x=0, y=0, origin=get_element_origin(target0))
        .pointer_down(button=0)
        .pointer_up(button=0)
    )
    (
        actions.add_pointer(input_id="finger2", pointer_type="touch")
        .pointer_move(x=0, y=0, origin=get_element_origin(target1))
        .pointer_down(button=0)
        .pointer_up(button=0)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], min_count=2)
    assert sorted(events, key=lambda event: event["target"]) == [
        {"target": "target0", "changedTouches": 2, "touches": 0},
        {"target": "target1", "changedTouches": 2, "touches": 0},
    ]
