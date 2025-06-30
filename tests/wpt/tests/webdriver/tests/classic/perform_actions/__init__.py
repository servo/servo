def perform_actions(session, actions):
    return session.transport.send(
        "POST",
        "/session/{session_id}/actions".format(session_id=session.session_id),
        {"actions": actions})


def assert_events(events, expected_events):
    events_not_seen = events.copy()

    for expected_event in expected_events:
       match_found = False
       for event in events_not_seen:
           # Check that all expected fields are present
           if all(item in event.items() for item in expected_event.items()):
               events_not_seen.remove(event)
               match_found = True
               break
       assert match_found, f"Expected event not found: {expected_event}"

    assert len(
        events_not_seen) == 0, f"Extra events received: {events_not_seen}"


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
