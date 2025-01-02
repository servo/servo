def assert_pointer_events(session, expected_events, target, pointer_type):
    events = session.execute_script("return window.recordedEvents;")
    assert len(events) == len(expected_events)
    event_types = [e["type"] for e in events]
    assert expected_events == event_types

    for e in events:
        assert e["target"] == target
        assert e["pointerType"] == pointer_type


def record_pointer_events(session, element):
    # Record basic mouse / pointer events on a given element.
    session.execute_script(
        """
        window.recordedEvents = [];
        function onPointerEvent(event) {
            window.recordedEvents.push({
                "pointerType": event.pointerType,
                "target": event.target.id,
                "type": event.type,
            });
        }
        arguments[0].addEventListener("pointerdown", onPointerEvent);
        arguments[0].addEventListener("pointerup", onPointerEvent);
    """,
        args=(element,),
    )
def perform_actions(session, actions):
    return session.transport.send(
        "POST",
        "/session/{session_id}/actions".format(session_id=session.session_id),
        {"actions": actions})
