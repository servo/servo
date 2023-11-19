from tests.support.asserts import assert_success
from tests.support.helpers import filter_dict

def get_events(session):
    """Return list of mouse events recorded in the fixture."""
    return session.execute_script("return allEvents.events;") or []

def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))

def test_event_mousemove(session, url):
    session.url = url(
        "/webdriver/tests/classic/element_click/support/test_click_wdspec.html"
    )

    element = session.find.css('#outer', all=False)
    response = element_click(session, element)
    assert_success(response)

    events = get_events(session)
    assert len(events) == 4

    expected = [
        {"type": "mousemove", "buttons": 0, "button": 0},
        {"type": "mousedown", "buttons": 1, "button": 0},
        {"type": "mouseup", "buttons": 0, "button": 0},
        {"type": "click", "buttons": 0, "button": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]

    assert expected == filtered_events
