import pytest

from tests.support.asserts import (
    assert_element_has_focus,
    assert_error,
    assert_same_element,
    assert_success,
)
from tests.support.inline import inline


def send_keys_to_element(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def add_event_listeners(element):
    element.session.execute_script("""
        window.events = [];
        var trackedEvents = ["focus", "change", "keypress", "keydown", "keyup", "input"];
        for (var i = 0; i < trackedEvents.length; i++) {
          arguments[0].addEventListener(trackedEvents[i], function(eventObject) { window.events.push(eventObject.type) });
        }
        """, args=(element,))


def get_events(session):
    return session.execute_script("return window.events")


def test_input(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    assert element.property("value") == ""

    send_keys_to_element(session, element, "foo")
    assert element.property("value") == "foo"
    assert_element_has_focus(element)


def test_textarea(session):
    session.url = inline("<textarea>")
    element = session.find.css("textarea", all=False)
    assert element.property("value") == ""

    send_keys_to_element(session, element, "foo")
    assert element.property("value") == "foo"
    assert_element_has_focus(element)


def test_input_append(session):
    session.url = inline("<input value=a>")
    element = session.find.css("input", all=False)
    assert element.property("value") == "a"

    send_keys_to_element(session, element, "b")
    assert element.property("value") == "ab"

    send_keys_to_element(session, element, "c")
    assert element.property("value") == "abc"


def test_textarea_append(session):
    session.url = inline("<textarea>a</textarea>")
    element = session.find.css("textarea", all=False)
    assert element.property("value") == "a"

    send_keys_to_element(session, element, "b")
    assert element.property("value") == "ab"

    send_keys_to_element(session, element, "c")
    assert element.property("value") == "abc"


@pytest.mark.parametrize("tag", ["input", "textarea"])
def test_events(session, tag):
    session.url = inline("<%s>" % tag)
    element = session.find.css(tag, all=False)
    add_event_listeners(element)

    send_keys_to_element(session, element, "foo")
    assert element.property("value") == "foo"
    assert get_events(session) == ["focus",
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
                                   "keyup"]


@pytest.mark.parametrize("tag", ["input", "textarea"])
def test_not_blurred(session, tag):
    session.url = inline("<%s>" % tag)
    element = session.find.css(tag, all=False)

    send_keys_to_element(session, element, "")
    assert_element_has_focus(element)
