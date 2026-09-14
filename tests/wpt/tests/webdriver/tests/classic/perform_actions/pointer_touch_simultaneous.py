import pytest

from tests.classic.perform_actions.support.refine import wait_for_events


@pytest.fixture
def setup_event_recorder(session, inline):
    def _setup_event_recorder(event_type):
        # "touch-action: none" keeps the synthesized touch pointers from being
        # consumed as a scroll gesture.
        session.url = inline(
            "<style>"
            "  html, body { margin: 0; touch-action: none; }"
            "  div { width: 200px; height: 200px; display: inline-block; }"
            "</style>"
            "<div id='target0'></div><div id='target1'></div>"
        )

        session.execute_script(
            """
            const eventType = arguments[0];
            window.allEvents = { events: [] };
            window.addEventListener(
                eventType,
                event => window.allEvents.events.push({
                    target: event.target.id,
                    changedTouches: event.changedTouches.length,
                    touches: event.touches.length,
                })
            );
            """,
            args=(event_type,),
        )

    return _setup_event_recorder


def test_two_pointer_ups_in_one_tick_are_simultaneous(session, setup_event_recorder):
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
    setup_event_recorder("touchend")

    target0 = session.find.css("#target0", all=False)
    target1 = session.find.css("#target1", all=False)

    finger1 = session.actions.sequence("pointer", "finger1", {"pointerType": "touch"})
    finger2 = session.actions.sequence("pointer", "finger2", {"pointerType": "touch"})
    finger1.pointer_move(0, 0, origin=target0) \
        .pointer_down(button=0) \
        .pointer_up(button=0)
    finger2.pointer_move(0, 0, origin=target1) \
        .pointer_down(button=0) \
        .pointer_up(button=0)
    session.actions.perform([finger1.dict, finger2.dict])

    events = wait_for_events(session, min_count=2)
    assert sorted(events, key=lambda event: event["target"]) == [
        {"target": "target0", "changedTouches": 2, "touches": 0},
        {"target": "target1", "changedTouches": 2, "touches": 0},
    ]
