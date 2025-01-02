import pytest

from webdriver import WebElement

from tests.support.asserts import assert_error, assert_success


@pytest.fixture
def check_doc():
    return """
    <input id=checked type=checkbox checked>
    <input id=notChecked type=checkbox>
    """


@pytest.fixture
def option_doc():
    return """
    <select>
      <option id=notSelected>r-
      <option id=selected selected>r+
    </select>
    """


def is_element_selected(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/selected".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window

    response = is_element_selected(session, element.id)
    assert_error(response, "no such window")
    response = is_element_selected(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = is_element_selected(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = is_element_selected(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = is_element_selected(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = is_element_selected(session, element.shadow_root.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = is_element_selected(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("input#text", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = is_element_selected(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, check_doc, as_frame):
    element = stale_element("input#checkbox", as_frame=as_frame)

    result = is_element_selected(session, element.id)
    assert_error(result, "stale element reference")


def test_element_checked(session, inline, check_doc):
    session.url = inline(check_doc)
    element = session.find.css("#checked", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, True)


def test_checkbox_not_selected(session, inline, check_doc):
    session.url = inline(check_doc)
    element = session.find.css("#notChecked", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, False)


def test_element_selected(session, inline, option_doc):
    session.url = inline(option_doc)
    element = session.find.css("#selected", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, True)


def test_element_not_selected(session, inline, option_doc):
    session.url = inline(option_doc)
    element = session.find.css("#notSelected", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, False)
