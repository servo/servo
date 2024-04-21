import pytest
from webdriver import WebElement

from tests.support.asserts import (
    assert_element_has_focus,
    assert_error,
    assert_events_equal,
    assert_in_events,
    assert_success,
)
from tests.support.dom import BUTTON_TYPES
from . import element_clear


@pytest.fixture
def tracked_events():
    return [
        "blur",
        "change",
        "focus",
    ]


@pytest.fixture(scope="session")
def text_file(tmpdir_factory):
    fh = tmpdir_factory.mktemp("tmp").join("hello.txt")
    fh.write("hello")
    return fh


def test_null_response_value(session, inline):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    element = WebElement(session, "foo")
    response = element_clear(session, element)
    assert_error(response, "no such window")

    original_handle, element = closed_window
    response = element_clear(session, element)
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = element_clear(session, element)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    element = WebElement(session, "foo")

    response = element_clear(session, element)
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = element_clear(session, element)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = element_clear(session, element.shadow_root)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = element_clear(session, element)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("div", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = element_clear(session, element)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    response = element_clear(session, element)
    assert_error(response, "stale element reference")


def test_pointer_interactable(session, inline):
    session.url = inline("<input style='margin-left: -1000px' value=foobar>")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "element not interactable")


def test_keyboard_interactable(session, inline):
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
def test_input(session, inline, add_event_listeners, tracked_events, type, value, default):
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
def test_input_readonly(session, inline, type):
    session.url = inline("<input type=%s readonly>" % type)
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_textarea(session, inline, add_event_listeners, tracked_events):
    session.url = inline("<textarea>foobar</textarea>")
    element = session.find.css("textarea", all=False)
    add_event_listeners(element, tracked_events)
    assert element.property("value") == "foobar"

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""
    assert_in_events(session, ["focus", "change", "blur"])


def test_textarea_readonly(session, inline):
    session.url = inline("<textarea readonly></textarea>")
    element = session.find.css("textarea", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_input_file(session, text_file, inline):
    session.url = inline("<input type=file>")
    element = session.find.css("input", all=False)
    element.send_keys(str(text_file))

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""


def test_input_file_multiple(session, text_file, inline):
    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)
    element.send_keys(str(text_file))
    element.send_keys(str(text_file))

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("value") == ""


@pytest.mark.parametrize("type", BUTTON_TYPES)
def test_button(session, inline, type):
    session.url = inline(f"""<button type="{type}">""")
    element = session.find.css("button", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_button_with_subtree(session, inline):
    """
    Elements inside button elements are interactable.
    """
    session.url = inline("""
        <button>
          <input value=foobar>
        </button>
        """)
    text_field = session.find.css("input", all=False)

    response = element_clear(session, text_field)
    assert_success(response)


def test_contenteditable(session, inline, add_event_listeners, tracked_events):
    session.url = inline("<p contenteditable>foobar</p>")
    element = session.find.css("p", all=False)
    add_event_listeners(element, tracked_events)
    assert element.property("innerHTML") == "foobar"

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("innerHTML") == ""
    assert_events_equal(session, ["focus", "blur"])
    assert_element_has_focus(session.execute_script("return document.body"))


def test_designmode(session, inline):
    session.url = inline("foobar")
    element = session.find.css("body", all=False)
    assert element.property("innerHTML") == "foobar"
    session.execute_script("document.designMode = 'on'")

    response = element_clear(session, element)
    assert_success(response)
    assert element.property("innerHTML") in ["", "<br>"]
    assert_element_has_focus(session.execute_script("return document.body"))


def test_resettable_element_focus_when_empty(session, inline, add_event_listeners, tracked_events):
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
def test_resettable_element_does_not_satisfy_validation_constraints(session, inline, type, invalid_value):
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
def test_non_editable_inputs(session, inline, type):
    session.url = inline("<input type=%s>" % type)
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_scroll_into_view(session, inline):
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
