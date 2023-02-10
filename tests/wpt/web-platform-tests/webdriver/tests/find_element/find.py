import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success


def find_element(session, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/element".format(**vars(session)),
        {"using": using, "value": value})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/element".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = find_element(session, "css selector", "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = find_element(session, "css selector", "foo")
    assert_error(response, "no such window")


@pytest.mark.parametrize(
    "selector",
    ["#same1", "#in-frame", "#in-shadow-root"],
    ids=["not-existent", "existent-other-frame", "existent-inside-shadow-root"],
)
def test_no_such_element_with_unknown_selector(session, get_test_page, selector):
    session.url = get_test_page()

    response = find_element(session, "css selector", selector)
    assert_error(response, "no such element")


@pytest.mark.parametrize("using", ["a", True, None, 1, [], {}])
def test_invalid_using_argument(session, using):
    response = find_element(session, using, "value")
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, [], {}])
def test_invalid_selector_argument(session, value):
    response = find_element(session, "css selector", value)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
def test_find_element(session, inline, using, value):
    session.url = inline("<a href=# id=linkText>full link text</a>")

    response = find_element(session, using, value)
    assert_success(response)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>link text</a>", "link text"),
    ("<a href=#>&nbsp;link text&nbsp;</a>", "link text"),
    ("<a href=#>link<br>text</a>", "link\ntext"),
    ("<a href=#>link&amp;text</a>", "link&text"),
    ("<a href=#>LINK TEXT</a>", "LINK TEXT"),
    ("<a href=# style='text-transform: uppercase'>link text</a>", "LINK TEXT"),
])
def test_find_element_link_text(session, inline, document, value):
    session.url = inline(document)

    response = find_element(session, "link text", value)
    assert_success(response)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>partial link text</a>", "link"),
    ("<a href=#>&nbsp;partial link text&nbsp;</a>", "link"),
    ("<a href=#>partial link text</a>", "k t"),
    ("<a href=#>partial link<br>text</a>", "k\nt"),
    ("<a href=#>partial link&amp;text</a>", "k&t"),
    ("<a href=#>PARTIAL LINK TEXT</a>", "LINK"),
    ("<a href=# style='text-transform: uppercase'>partial link text</a>", "LINK"),
])
def test_find_element_partial_link_text(session, inline, document, value):
    session.url = inline(document)

    response = find_element(session, "partial link text", value)
    assert_success(response)


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//*[name()='a']")])
def test_xhtml_namespace(session, inline, using, value):
    session.url = inline("""<a href="#" id="linkText">full link text</a>""",
                         doctype="xhtml")
    expected = session.execute_script("return document.links[0]")

    response = find_element(session, using, value)
    value = assert_success(response)
    assert_same_element(session, value, expected)


@pytest.mark.parametrize("using,value",
                         [("css selector", ":root"),
                          ("tag name", "html"),
                          ("xpath", "/html")])
def test_htmldocument(session, inline, using, value):
    session.url = inline("")
    response = find_element(session, using, value)
    assert_success(response)
