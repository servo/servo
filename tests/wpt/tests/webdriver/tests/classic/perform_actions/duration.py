import pytest

from tests.classic.perform_actions.support.refine import wait_for_events


# A move or scroll action with a duration greater than zero has to be split into
# multiple incremental events.
DURATION = 200


@pytest.fixture
def setup_event_recorder(session, inline):
    def _setup_event_recorder(event_type):
        # "touch-action: none" keeps a synthesized touch drag from being
        # consumed as a scroll gesture, so the incremental touch events are
        # delivered to the content as "pointermove" events instead of a
        # "pointercancel".
        session.url = inline(
            "<style>:root { touch-action: none }</style>"
            "<body style='width: 1000px; height: 500px'></body>"
        )

        session.execute_script(
            """
            const eventType = arguments[0];
            window.allEvents = { events: [] };
            window.addEventListener(
                eventType,
                event => window.allEvents.events.push({
                    type: event.type,
                    clientX: event.clientX,
                    clientY: event.clientY,
                    deltaX: event.deltaX,
                    deltaY: event.deltaY,
                })
            );
            """,
            args=(event_type,),
        )

    return _setup_event_recorder


def test_mouse_move_dispatches_multiple_events(session, setup_event_recorder, mouse_chain):
    setup_event_recorder("mousemove")

    mouse_chain \
        .pointer_move(100, 200, duration=DURATION) \
        .perform()

    events = wait_for_events(session, min_count=2)
    assert len(events) > 1
    assert all(event["type"] == "mousemove" for event in events)

    # The last event has to reach the requested target position.
    assert events[-1]["clientX"] == pytest.approx(100, abs=1.0)
    assert events[-1]["clientY"] == pytest.approx(200, abs=1.0)


def test_touch_move_dispatches_multiple_events(session, setup_event_recorder, touch_chain):
    setup_event_recorder("pointermove")

    touch_chain \
        .pointer_down() \
        .pointer_move(200, 100, duration=DURATION) \
        .pointer_up() \
        .perform()

    events = wait_for_events(session, min_count=2)
    assert len(events) > 1
    assert all(event["type"] == "pointermove" for event in events)

    # The last event has to reach the requested target position.
    assert events[-1]["clientX"] == pytest.approx(200, abs=1.0)
    assert events[-1]["clientY"] == pytest.approx(100, abs=1.0)


def test_wheel_scroll_dispatches_multiple_events(session, setup_event_recorder, wheel_chain):
    setup_event_recorder("wheel")

    wheel_chain.scroll(0, 0, 30, 60, duration=DURATION).perform()

    events = wait_for_events(session, min_count=2)
    assert len(events) > 1
    assert all(event["type"] == "wheel" for event in events)

    # The incremental deltas of all dispatched events have to add up to the
    # requested scroll distance.
    assert sum(event["deltaX"] for event in events) == 30
    assert sum(event["deltaY"] for event in events) == 60
