import pytest
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success


def find_element(session, shadow_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/shadow/{shadow_id}/element".format(
            session_id=session.session_id,
            shadow_id=shadow_id),
        {"using": using, "value": value})


def test_null_parameter_value(session, http, inline, get_shadow_page):
    session.url = inline(get_shadow_page("<div><a href=# id=linkText>full link text</a></div>"))
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root

    path = "/session/{session_id}/shadow/{shadow_id}/element".format(
        session_id=session.session_id, shadow_id=shadow_root.id)
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = find_element(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = find_element(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_unknown_shadow_root(session, inline, get_shadow_page):
    session.url = inline(get_shadow_page("<div><input type='checkbox'/></div>"))
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root

    session.url = inline("<p>")

    result = find_element(session, shadow_root.id, "css selector", "input")
    assert_error(result, "no such element")


@pytest.mark.parametrize(
    "value",
    ["#doesNotExist", "#inner"],
    ids=["not-existent", "existent-inner-shadow-dom"],
)
def test_no_such_element_with_invalid_value(
    session, iframe, inline, get_shadow_page, value
):
    session.url = inline(get_shadow_page(f"""
        <div id="outer"/>
        {get_shadow_page("<div id='inner'>")}
    """))

    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root

    response = find_element(session, shadow_root.id, "css selector", value)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root_from_other_window_handle(
    session, inline, get_shadow_page
):
    session.url = inline(get_shadow_page("<div>"))
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = find_element(session, shadow_root.id, "css selector", "div")
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root_from_other_frame(
    session, iframe, inline, get_shadow_page
):
    session.url = inline(iframe(get_shadow_page("<div>")))

    session.switch_frame(0)
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    session.switch_frame("parent")

    response = find_element(session, shadow_root.id, "css selector", "div")
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_detached_shadow_root(session, iframe, inline, get_shadow_page, as_frame):
    page = get_shadow_page("<div><input type='checkbox'/></div>")

    if as_frame:
        session.url = inline(iframe(page))
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)
    else:
        session.url = inline(page)

    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root

    session.execute_script("arguments[0].remove();", args=[custom_element])

    response = find_element(session, shadow_root.id, "css selector", "input")
    assert_error(response, "detached shadow root")


@pytest.mark.parametrize("using", ["a", True, None, 1, [], {}])
def test_invalid_using_argument(session, using):
    response = find_element(session, "notReal", using, "value")
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, [], {}])
def test_invalid_selector_argument(session, value):
    response = find_element(session, "notReal", "css selector", value)
    assert_error(response, "invalid argument")


def test_found_element_equivalence(session, inline, get_shadow_page):
    session.url = inline(get_shadow_page("<div><input type='checkbox'/></div>"))
    custom_element = session.find.css("custom-shadow-element", all=False)
    expected = session.execute_script("return arguments[0].shadowRoot.querySelector('input')",
                                      args=(custom_element,))
    shadow_root = custom_element.shadow_root
    response = find_element(session, shadow_root.id, "css selector", "input")
    value = assert_success(response)
    assert_same_element(session, value, expected)


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
def test_find_element(session, inline, get_shadow_page, using, value):
    session.url = inline(get_shadow_page("<div><a href=# id=linkText>full link text</a></div>"))
    custom_element = session.find.css("custom-shadow-element", all=False)
    expected = session.execute_script("return arguments[0].shadowRoot.querySelector('#linkText')",
                                      args=(custom_element,))
    shadow_root = custom_element.shadow_root
    response = find_element(session, shadow_root.id, using, value)
    value = assert_success(response)
    assert_same_element(session, value, expected)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>link text</a>", "link text"),
    ("<a href=#>&nbsp;link text&nbsp;</a>", "link text"),
    ("<a href=#>link<br>text</a>", "link\ntext"),
    ("<a href=#>link&amp;text</a>", "link&text"),
    ("<a href=#>LINK TEXT</a>", "LINK TEXT"),
    ("<a href=# style='text-transform: uppercase'>link text</a>", "LINK TEXT"),
])
def test_find_element_link_text(session, inline, get_shadow_page, document, value):
    session.url = inline(get_shadow_page("<div>{0}</div>".format(document)))
    custom_element = session.find.css("custom-shadow-element", all=False)
    expected = session.execute_script("return arguments[0].shadowRoot.querySelectorAll('a')[0]",
                                      args=(custom_element,))
    shadow_root = custom_element.shadow_root

    response = find_element(session, shadow_root.id, "link text", value)
    value = assert_success(response)
    assert_same_element(session, value, expected)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>partial link text</a>", "link"),
    ("<a href=#>&nbsp;partial link text&nbsp;</a>", "link"),
    ("<a href=#>partial link text</a>", "k t"),
    ("<a href=#>partial link<br>text</a>", "k\nt"),
    ("<a href=#>partial link&amp;text</a>", "k&t"),
    ("<a href=#>PARTIAL LINK TEXT</a>", "LINK"),
    ("<a href=# style='text-transform: uppercase'>partial link text</a>", "LINK"),
])
def test_find_element_partial_link_text(session, inline, get_shadow_page, document, value):
    session.url = inline(get_shadow_page("<div>{0}</div>".format(document)))
    custom_element = session.find.css("custom-shadow-element", all=False)
    expected = session.execute_script("return arguments[0].shadowRoot.querySelectorAll('a')[0]",
                                      args=(custom_element,))
    shadow_root = custom_element.shadow_root

    response = find_element(session, shadow_root.id, "partial link text", value)
    value = assert_success(response)
    assert_same_element(session, value, expected)
