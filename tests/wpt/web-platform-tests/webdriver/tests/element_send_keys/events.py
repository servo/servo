import pytest

from tests.support.asserts import (
    assert_element_has_focus,
    assert_events_equal,
    assert_success,
)

from tests.support.inline import inline


@pytest.fixture
def tracked_events():
    return [
        "blur",
        "change",
        "focus",
        "input",
        "keydown",
        "keypress",
        "keyup",
    ]


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def test_file_upload(session, create_file, add_event_listeners, tracked_events):
    expected_events = [
        "input",
        "change",
    ]

    single_file = create_file("foo")

    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)
    add_event_listeners(element, tracked_events)

    response = element_send_keys(session, element, str(single_file))
    assert_success(response)

    assert_events_equal(session, expected_events)


@pytest.mark.parametrize("tag", ["input", "textarea"])
def test_form_control_send_text(session, add_event_listeners, tracked_events, tag):
    expected_events = [
        "focus",
        "keydown",
        "keypress",
        "input",
        "keyup",
        "keydown",
        "keypress",
        "input",
        "keyup",
        "keydown",
        "keypress",
        "input",
        "keyup",
    ]

    session.url = inline("<%s>" % tag)
    element = session.find.css(tag, all=False)
    add_event_listeners(element, tracked_events)

    response = element_send_keys(session, element, "foo")
    assert_success(response)
    assert_events_equal(session, expected_events)


@pytest.mark.parametrize("tag", ["input", "textarea"])
def test_not_blurred(session, tag):
    session.url = inline("<%s>" % tag)
    element = session.find.css(tag, all=False)

    response = element_send_keys(session, element, "")
    assert_success(response)
    assert_element_has_focus(element)
