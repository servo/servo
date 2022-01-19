import pytest

from tests.support.asserts import assert_error, assert_success


def get_element_attribute(session, element, attr):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/attribute/{attr}".format(
            session_id=session.session_id,
            element_id=element,
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


def test_element_not_found(session):
    # 13.2 Step 3
    result = get_element_attribute(session, "foo", "id")
    assert_error(result, "no such element")


def test_element_stale(session, inline):
    session.url = inline("<input id=foo>")
    element = session.find.css("input", all=False)
    session.refresh()
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
