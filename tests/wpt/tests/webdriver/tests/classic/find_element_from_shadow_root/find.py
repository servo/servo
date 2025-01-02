import pytest
from webdriver.client import WebElement, ShadowRoot
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success


def find_element(session, shadow_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/shadow/{shadow_id}/element".format(
            session_id=session.session_id,
            shadow_id=shadow_id),
        {"using": using, "value": value})


def test_null_parameter_value(session, http, get_test_page):
    session.url = get_test_page()

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

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


def test_no_such_shadow_root_with_element(session, get_test_page):
    session.url = get_test_page()

    host = session.find.css("custom-element", all=False)

    result = find_element(session, host.id, "css selector", "input")
    assert_error(result, "no such shadow root")


def test_no_such_shadow_root_with_unknown_shadow_root(session):
    shadow_root = ShadowRoot(session, "foo")

    result = find_element(session, shadow_root.id, "css selector", "input")
    assert_error(result, "no such shadow root")


def test_no_such_shadow_root_with_shadow_root_from_other_window_handle(
    session, get_test_page
):
    session.url = get_test_page()

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = find_element(session, shadow_root.id, "css selector", "div")
    assert_error(response, "no such shadow root")


def test_no_such_shadow_root_with_shadow_root_from_other_frame(
    session, get_test_page
):
    session.url = get_test_page(as_frame=True)
    session.switch_frame(0)

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    session.switch_frame("parent")

    response = find_element(session, shadow_root.id, "css selector", "div")
    assert_error(response, "no such shadow root")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_detached_shadow_root(session, get_test_page, as_frame):
    session.url = get_test_page(as_frame=as_frame)

    if as_frame:
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    session.execute_script("arguments[0].remove();", args=[host])

    response = find_element(session, shadow_root.id, "css selector", "input")
    assert_error(response, "detached shadow root")


@pytest.mark.parametrize(
    "selector",
    ["#same1", "#in-frame", "#with-children"],
    ids=["not-existent", "existent-other-frame", "existent-outside-shadow-root"],
)
def test_no_such_element_with_unknown_selector(session, get_test_page, selector):
    session.url = get_test_page()

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    response = find_element(session, shadow_root.id, "css selector", selector)
    assert_error(response, "no such element")


@pytest.mark.parametrize("shadow_root_id", [True, None, 1, [], {}])
def test_invalid_shadow_root_id_argument(session, get_test_page, shadow_root_id):
    session.url = get_test_page()

    response = find_element(session, shadow_root_id, "css selector", "input")
    assert_error(response, "no such shadow root")


@pytest.mark.parametrize("using", ["a", True, None, 1, [], {}])
def test_invalid_using_argument(session, get_test_page, using):
    session.url = get_test_page()
    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    response = find_element(session, shadow_root.id, using, "input")
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, [], {}])
def test_invalid_selector_argument(session, get_test_page, value):
    session.url = get_test_page()
    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    response = find_element(session, shadow_root.id, "css selector", value)
    assert_error(response, "invalid argument")


def test_found_element_equivalence(session, get_test_page):
    session.url = get_test_page()

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    expected = session.execute_script("""
        return arguments[0].shadowRoot.querySelector('input')
        """, args=(host,))

    response = find_element(session, shadow_root.id, "css selector", "input")
    value = assert_success(response)
    assert_same_element(session, value, expected)


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
@pytest.mark.parametrize("mode", ["open", "closed"])
def test_find_element(session, get_test_page, using, value, mode):
    expected_text = "full link text"
    session.url = get_test_page(
        shadow_doc=f"<div><a href=# id=linkText>{expected_text}</a></div>",
        shadow_root_mode=mode,
    )
    shadow_root = session.find.css("custom-element", all=False).shadow_root

    result = find_element(session, shadow_root.id, using, value)
    value = assert_success(result)

    element = WebElement.from_json(value, session)
    assert element.text == expected_text


@pytest.mark.parametrize("document,value", [
    ("<a href=#>link text</a>", "link text"),
    ("<a href=#>&nbsp;link text&nbsp;</a>", "link text"),
    ("<a href=#>link<br>text</a>", "link\ntext"),
    ("<a href=#>link&amp;text</a>", "link&text"),
    ("<a href=#>LINK TEXT</a>", "LINK TEXT"),
    ("<a href=# style='text-transform: uppercase'>link text</a>", "LINK TEXT"),
])
def test_find_element_link_text(session, get_test_page, document, value):
    session.url = get_test_page(shadow_doc=f"<div>{document}</div>")

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    expected = session.execute_script("""
        return arguments[0].shadowRoot.querySelectorAll('a')[0]
        """, args=(host,))

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
def test_find_element_partial_link_text(session, get_test_page, document, value):
    session.url = get_test_page(shadow_doc=f"<div>{document}</div>")

    host = session.find.css("custom-element", all=False)
    shadow_root = host.shadow_root

    expected = session.execute_script("""
        return arguments[0].shadowRoot.querySelectorAll('a')[0]
        """, args=(host,))

    response = find_element(session, shadow_root.id, "partial link text", value)
    value = assert_success(response)
    assert_same_element(session, value, expected)


@pytest.mark.parametrize("mode", ["open", "closed"])
def test_find_element_in_nested_shadow_root(session, get_test_page, mode):
    expected_text = "full link text"
    session.url = get_test_page(
        shadow_doc=f"<div><a href=# id=linkText>{expected_text}</a></div>",
        shadow_root_mode=mode,
        nested_shadow_dom=True,
    )
    shadow_root = session.find.css("custom-element", all=False).shadow_root

    inner_custom_element = shadow_root.find_element(
        "css selector", "inner-custom-element"
    )
    nested_shadow_root = inner_custom_element.shadow_root

    result = find_element(session, nested_shadow_root.id, "css selector", "#linkText")
    value = assert_success(result)

    element = WebElement.from_json(value, session)
    assert element.text == expected_text
