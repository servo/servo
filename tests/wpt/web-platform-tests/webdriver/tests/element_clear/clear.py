# META: timeout=long

import pytest

from tests.support.asserts import (
    assert_element_has_focus,
    assert_error,
    assert_events_equal,
    assert_in_events,
    assert_success,
)
from tests.support.inline import inline


@pytest.fixture
def tracked_events():
    return [
        "blur",
        "change",
        "focus",
    ]


def element_clear(session, element):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/clear".format(
            session_id=session.session_id,
            element_id=element.id))


@pytest.fixture(scope="session")
def text_file(tmpdir_factory):
    fh = tmpdir_factory.mktemp("tmp").join("hello.txt")
    fh.write("hello")
    return fh


def test_null_response_value(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    value = assert_success(response)
    assert value is None


def test_closed_context(session, create_window):
    new_window = create_window()
    session.window_handle = new_window
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    session.close()

    response = element_clear(session, element)
    assert_error(response, "no such window")


def test_connected_element(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    session.url = inline("<input>")
    response = element_clear(session, element)
    assert_error(response, "stale element reference")


def test_pointer_interactable(session):
    session.url = inline("<input style='margin-left: -1000px' value=foobar>")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "element not interactable")


def test_keyboard_interactable(session):
    session.url = inline("""
        <input value=foobar>
        <div></div>

        <style>
        div {
          position: absolute;
          background: blue;
          top: 0;
        }
        </style>
        """)
    element = session.find.css("input", all=False)
    assert element.property("value") == "foobar"

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""


@pytest.mark.parametrize("type,value,default",
                         [("number", "42", ""),
                          ("range", "42", "50"),
                          ("email", "foo@example.com", ""),
                          ("password", "password", ""),
                          ("search", "search", ""),
                          ("tel", "999", ""),
                          ("text", "text", ""),
                          ("url", "https://example.com/", ""),
                          ("color", "#ff0000", "#000000"),
                          ("date", "2017-12-26", ""),
                          ("datetime", "2017-12-26T19:48", ""),
                          ("datetime-local", "2017-12-26T19:48", ""),
                          ("time", "19:48", ""),
                          ("month", "2017-11", ""),
                          ("week", "2017-W52", "")])
def test_input(session, add_event_listeners, tracked_events, type, value, default):
    session.url = inline("<input type=%s value='%s'>" % (type, value))
    element = session.find.css("input", all=False)
    add_event_listeners(element, tracked_events)
    assert element.property("value") == value

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == default
    assert_in_events(session, ["focus", "change", "blur"])
    assert_element_has_focus(session.execute_script("return document.body"))


@pytest.mark.parametrize("type",
                         ["number",
                          "range",
                          "email",
                          "password",
                          "search",
                          "tel",
                          "text",
                          "url",
                          "color",
                          "date",
                          "datetime",
                          "datetime-local",
                          "time",
                          "month",
                          "week",
                          "file"])
def test_input_disabled(session, type):
    session.url = inline("<input type=%s disabled>" % type)
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


@pytest.mark.parametrize("type",
                         ["number",
                          "range",
                          "email",
                          "password",
                          "search",
                          "tel",
                          "text",
                          "url",
                          "color",
                          "date",
                          "datetime",
                          "datetime-local",
                          "time",
                          "month",
                          "week",
                          "file"])
def test_input_readonly(session, type):
    session.url = inline("<input type=%s readonly>" % type)
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_textarea(session, add_event_listeners, tracked_events):
    session.url = inline("<textarea>foobar</textarea>")
    element = session.find.css("textarea", all=False)
    add_event_listeners(element, tracked_events)
    assert element.property("value") == "foobar"

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""
    assert_in_events(session, ["focus", "change", "blur"])


def test_textarea_disabled(session):
    session.url = inline("<textarea disabled></textarea>")
    element = session.find.css("textarea", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_textarea_readonly(session):
    session.url = inline("<textarea readonly></textarea>")
    element = session.find.css("textarea", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_input_file(session, text_file):
    session.url = inline("<input type=file>")
    element = session.find.css("input", all=False)
    element.send_keys(str(text_file))

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""


def test_input_file_multiple(session, text_file):
    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)
    element.send_keys(str(text_file))
    element.send_keys(str(text_file))

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""


def test_select(session):
    session.url = inline("""
        <select>
          <option>foo
        </select>
        """)
    select = session.find.css("select", all=False)
    option = session.find.css("option", all=False)

    response = element_clear(session, select)
    assert_error(response, "invalid element state")
    response = element_clear(session, option)
    assert_error(response, "invalid element state")


def test_button(session):
    session.url = inline("<button></button>")
    button = session.find.css("button", all=False)

    response = element_clear(session, button)
    assert_error(response, "invalid element state")


def test_button_with_subtree(session):
    """
    Whilst an <input> is normally editable, the focusable area
    where it is placed will default to the <button>.  I.e. if you
    try to click <input> to focus it, you will hit the <button>.
    """
    session.url = inline("""
        <button>
          <input value=foobar>
        </button>
        """)
    text_field = session.find.css("input", all=False)

    response = element_clear(session, text_field)
    assert_error(response, "element not interactable")


def test_contenteditable(session, add_event_listeners, tracked_events):
    session.url = inline("<p contenteditable>foobar</p>")
    element = session.find.css("p", all=False)
    add_event_listeners(element, tracked_events)
    assert element.property("innerHTML") == "foobar"

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("innerHTML") == ""
    assert_events_equal(session, ["focus", "change", "blur"])
    assert_element_has_focus(session.execute_script("return document.body"))


def test_designmode(session):
    session.url = inline("foobar")
    element = session.find.css("body", all=False)
    assert element.property("innerHTML") == "foobar"
    session.execute_script("document.designMode = 'on'")

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("innerHTML") == "<br>"
    assert_element_has_focus(session.execute_script("return document.body"))


def test_resettable_element_focus_when_empty(session, add_event_listeners, tracked_events):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    add_event_listeners(element, tracked_events)
    assert element.property("value") == ""

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""
    assert_events_equal(session, [])


@pytest.mark.parametrize("type,invalid_value",
                         [("number", "foo"),
                          ("range", "foo"),
                          ("email", "foo"),
                          ("url", "foo"),
                          ("color", "foo"),
                          ("date", "foo"),
                          ("datetime", "foo"),
                          ("datetime-local", "foo"),
                          ("time", "foo"),
                          ("month", "foo"),
                          ("week", "foo")])
def test_resettable_element_does_not_satisfy_validation_constraints(session, type, invalid_value):
    """
    Some UAs allow invalid input to certain types of constrained
    form controls.  For example, Gecko allows non-valid characters
    to be typed into <input type=number> but Chrome does not.
    Since we want to test that Element Clear works for clearing the
    invalid characters in these UAs, it is fine to skip this test
    where UAs do not allow the element to not satisfy its constraints.
    """
    session.url = inline("<input type=%s>" % type)
    element = session.find.css("input", all=False)

    def is_valid(element):
        return session.execute_script("""
            var input = arguments[0];
            return input.validity.valid;
            """, args=(element,))

    # value property does not get updated if the input is invalid
    element.send_keys(invalid_value)

    # UA does not allow invalid input for this form control type
    if is_valid(element):
        return

    response = element_clear(session, element)
    assert_success(response)
    assert is_valid(element)


@pytest.mark.parametrize("type",
                         ["checkbox",
                          "radio",
                          "hidden",
                          "submit",
                          "button",
                          "image"])
def test_non_editable_inputs(session, type):
    session.url = inline("<input type=%s>" % type)
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_scroll_into_view(session):
    session.url = inline("""
        <input value=foobar>
        <div style='height: 200vh; width: 5000vh'>
        """)
    element = session.find.css("input", all=False)
    assert element.property("value") == "foobar"
    assert session.execute_script("return window.pageYOffset") == 0

    # scroll to the bottom right of the page
    session.execute_script("""
        var body = document.body;
        window.scrollTo(body.scrollWidth, body.scrollHeight);
        """)

    # clear and scroll back to the top of the page
    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""

    # check if element cleared is scrolled into view
    rect = session.execute_script("""
        var input = arguments[0];
        var rect = input.getBoundingClientRect();
        return {"top": rect.top,
                "left": rect.left,
                "height": rect.height,
                "width": rect.width};
        """, args=(element,))
    window = session.execute_script("""
        return {"innerHeight": window.innerHeight,
                "innerWidth": window.innerWidth,
                "pageXOffset": window.pageXOffset,
                "pageYOffset": window.pageYOffset};
        """)

    assert rect["top"] < (window["innerHeight"] + window["pageYOffset"]) and \
           rect["left"] < (window["innerWidth"] + window["pageXOffset"]) and \
           (rect["top"] + element.rect["height"]) > window["pageYOffset"] and \
           (rect["left"] + element.rect["width"]) > window["pageXOffset"]
