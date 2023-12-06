import pytest

from webdriver import WebElement

from tests.support.asserts import assert_error, assert_success


def get_computed_role(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/computedrole".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_browsing_context(session, closed_frame):
    response = get_computed_role(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    result = get_computed_role(session, element.id)
    assert_error(result, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = get_computed_role(session, element.shadow_root.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    result = get_computed_role(session, element.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("div", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    result = get_computed_role(session, element.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    response = get_computed_role(session, element.id)
    assert_error(response, "stale element reference")


@pytest.mark.parametrize("html,tag,expected", [
    ("<article>foo</article>", "article", "article"),
    ("<input role=searchbox>", "input", "searchbox"),
    ("<img role=button tabindex=0>", "img", "button")])
def test_computed_roles(session, inline, html, tag, expected):
    session.url = inline(html)
    element = session.find.css(tag, all=False)
    result = get_computed_role(session, element.id)
    assert_success(result, expected)
