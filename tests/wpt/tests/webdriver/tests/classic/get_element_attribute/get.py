import pytest

from webdriver import WebElement

from tests.support.asserts import assert_error, assert_success


def get_element_attribute(session, element_id, attr):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/attribute/{attr}".format(
            session_id=session.session_id,
            element_id=element_id,
            attr=attr))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_attribute(session, element.id, "id")
    assert_error(response, "no such window")
    response = get_element_attribute(session, "foo", "id")
    assert_error(response, "no such window")
    session.window_handle = original_handle
    response = get_element_attribute(session, element.id, "id")
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_attribute(session, "foo", "id")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = get_element_attribute(session, element.id, "id")
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = get_element_attribute(session, element.shadow_root.id, "id")
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = get_element_attribute(session, element.id, "id")
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

    response = get_element_attribute(session, element.id, "id")
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    result = get_element_attribute(session, element.id, "id")
    assert_error(result, "stale element reference")


def test_normal(session, inline):
    # 13.2 Step 5
    session.url = inline("<input type=checkbox>")
    element = session.find.css("input", all=False)
    result = get_element_attribute(session, element.id, "input")
    assert_success(result, None)

    # Check we are not returning the property which will have a different value
    assert session.execute_script("return document.querySelector('input').checked") is False
    element.click()
    assert session.execute_script("return document.querySelector('input').checked") is True
    result = get_element_attribute(session, element.id, "input")
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
    ("input", [
        "autofocus", "checked", "disabled", "formnovalidate", "multiple", "readonly", "required"
    ]),
    ("menuitem", ["checked", "default", "disabled"]),
    ("ol", ["reversed"]),
    ("optgroup", ["disabled"]),
    ("option", ["disabled", "selected"]),
    ("script", ["async", "defer"]),
    ("select", ["autofocus", "disabled", "multiple", "required"]),
    ("textarea", ["autofocus", "disabled", "readonly", "required"]),
    ("track", ["default"]),
    ("video", ["autoplay", "controls", "loop", "muted"])
])
def test_boolean_attribute(session, inline, tag, attrs):
    for attr in attrs:
        session.url = inline("<{0} {1}>".format(tag, attr))
        element = session.find.css(tag, all=False)
        result = get_element_attribute(session, element.id, attr)
        assert_success(result, "true")


def test_global_boolean_attributes(session, inline):
    session.url = inline("<p hidden>foo")
    element = session.find.css("p", all=False)
    result = get_element_attribute(session, element.id, "hidden")

    assert_success(result, "true")

    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)
    result = get_element_attribute(session, element.id, "hidden")
    assert_success(result, None)

    session.url = inline("<p itemscope>foo")
    element = session.find.css("p", all=False)
    result = get_element_attribute(session, element.id, "itemscope")

    assert_success(result, "true")

    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)
    result = get_element_attribute(session, element.id, "itemscope")
    assert_success(result, None)


@pytest.mark.parametrize("is_relative", [True, False], ids=["relative", "absolute"])
def test_anchor_href(session, inline, url, is_relative):
    href = "/foo.html" if is_relative else url("/foo.html")

    session.url = inline("<a href='{}'>foo</a>".format(href))
    element = session.find.css("a", all=False)

    response = get_element_attribute(session, element.id, "href")
    assert_success(response, href)
