# META: timeout=long

import pytest

from tests.support.asserts import assert_error, assert_success, assert_dialog_handled
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


def get_attribute(session, element, attr):
    return session.transport.send("GET", "session/{session_id}/element/{element_id}/attribute/{attr}"
                                  .format(session_id=session.session_id,
                                          element_id=element,
                                          attr=attr))


# 13.2 Get Element Attribute

def test_no_browsing_context(session, create_window):
    # 13.2 step 1
    session.window_handle = create_window()
    session.close()

    result = get_attribute(session, "foo", "id")

    assert_error(result, "no such window")


def test_handle_prompt_dismiss(new_session, add_browser_capabilites):
    # 13.2 step 2
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss"})}})
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = get_attribute(session, element.id, "id")

    assert_success(result, "foo")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = get_attribute(session, element.id, "id")

    assert_success(result, "foo")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = get_attribute(session, element.id, "id")

    assert_success(result, "foo")
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    # 13.2 step 2
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = get_attribute(session, element.id, "id")

    assert_success(result, "foo")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = get_attribute(session, element.id, "id")

    assert_success(result, "foo")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = get_attribute(session, element.id, "id")

    assert_success(result, "foo")
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_missing_value(session):
    # 13.2 step 2
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = get_attribute(session, element.id, "id")

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = get_attribute(session, element.id, "id")

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = get_attribute(session, element.id, "id")

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")


def test_element_not_found(session):
    # 13.2 Step 3
    result = get_attribute(session, "foo", "id")

    assert_error(result, "no such element")


def test_element_stale(session):
    # 13.2 step 4
    session.url = inline("<input id=foo>")
    element = session.find.css("input", all=False)
    session.refresh()
    result = get_attribute(session, element.id, "id")

    assert_error(result, "stale element reference")


def test_normal(session):
    # 13.2 Step 5
    session.url = inline("<input type=checkbox>")
    element = session.find.css("input", all=False)
    result = get_attribute(session, element.id, "input")
    assert_success(result, None)
    assert False == session.execute_script("return document.querySelector('input').checked")

    # Check we are not returning the property which will have a different value
    element.click()
    assert True == session.execute_script("return document.querySelector('input').checked")
    result = get_attribute(session, element.id, "input")
    assert_success(result, None)


@pytest.mark.parametrize("tag,attrs", [
    ("audio", ["autoplay", "controls", "loop", "muted"]),
    ("button", ["autofocus", "disabled", "formnovalidate"]),
    ("details", ["open"]),
    ("dialog", ["open"]),
    ("fieldset", ["disabled"]),
    ("form", ["novalidate"]),
    ("iframe", ["allowfullscreen"]),
    ("img", ["ismap"]),
    ("input", ["autofocus", "checked", "disabled", "formnovalidate", "multiple", "readonly", "required"]),
    ("menuitem", ["checked", "default", "disabled"]),
    ("object", ["typemustmatch"]),
    ("ol", ["reversed"]),
    ("optgroup", ["disabled"]),
    ("option", ["disabled", "selected"]),
    ("script", ["async", "defer"]),
    ("select", ["autofocus", "disabled", "multiple", "required"]),
    ("textarea", ["autofocus", "disabled", "readonly", "required"]),
    ("track", ["default"]),
    ("video", ["autoplay", "controls", "loop", "muted"])
])
def test_boolean_attribute(session, tag, attrs):
    # 13.2 Step 5
    for attr in attrs:
        session.url = inline("<{0} {1}>".format(tag, attr))

        element = session.find.css(tag, all=False)
        result = result = get_attribute(session, element.id, attr)
        assert_success(result, "true")


def test_global_boolean_attributes(session):
    # 13.2 Step 5
    session.url = inline("<p hidden>foo")
    element = session.find.css("p", all=False)
    result = result = get_attribute(session, element.id, "hidden")

    assert_success(result, "true")

    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)
    result = result = get_attribute(session, element.id, "hidden")
    assert_success(result, None)

    session.url = inline("<p itemscope>foo")
    element = session.find.css("p", all=False)
    result = result = get_attribute(session, element.id, "itemscope")

    assert_success(result, "true")

    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)
    result = result = get_attribute(session, element.id, "itemscope")
    assert_success(result, None)
