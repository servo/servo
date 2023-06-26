import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success


def find_elements(session, element_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/elements".format(
            session_id=session.session_id,
            element_id=element_id),
        {"using": using, "value": value})


def test_null_parameter_value(session, http, inline):
    session.url = inline("<div><a href=# id=linkText>full link text</a></div>")
    element = session.find.css("div", all=False)

    path = "/session/{session_id}/element/{element_id}/elements".format(
        session_id=session.session_id, element_id=element.id)
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = find_elements(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = find_elements(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = find_elements(session, element.shadow_root.id, "css selector", "#in-shadow-dom")
    assert_error(result, "no such element")


@pytest.mark.parametrize(
    "selector",
    ["#same1", "#in-frame", "#in-shadow-dom"],
    ids=["not-existent", "existent-other-frame", "existent-inside-shadow-root"],
)
def test_no_elements_with_unknown_selector(session, get_test_page,selector):
    session.url = get_test_page()

    element = session.find.css(":root", all=False)
    response = find_elements(session, element.id, "css selector", selector)
    elements = assert_success(response)
    assert elements == []


def test_no_such_element_with_startnode_from_other_window_handle(session, inline):
    session.url = inline("<div id='parent'><p/>")
    from_element = session.find.css("#parent", all=False)

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = find_elements(session, from_element.id, "css selector", "p")
    assert_error(response, "no such element")


def test_no_such_element_with_startnode_from_other_frame(session, iframe, inline):
    session.url = inline(iframe("<div id='parent'><p/>"))

    session.switch_frame(0)
    from_element = session.find.css("#parent", all=False)
    session.switch_frame("parent")

    response = find_elements(session, from_element.id, "css selector", "p")
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("div#with-children", as_frame=as_frame)

    response = find_elements(session, element.id, "css selector", "p")
    assert_error(response, "stale element reference")


@pytest.mark.parametrize("using", [("a"), (True), (None), (1), ([]), ({})])
def test_invalid_using_argument(session, using):
    response = find_elements(session, "notReal", using, "value")
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, [], {}])
def test_invalid_selector_argument(session, value):
    response = find_elements(session, "notReal", "css selector", value)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
def test_find_elements(session, inline, using, value):
    session.url = inline("<div><a href=# id=linkText>full link text</a></div>")
    element = session.find.css("div", all=False)
    response = find_elements(session, element.id, using, value)
    assert_success(response)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>link text</a>", "link text"),
    ("<a href=#>&nbsp;link text&nbsp;</a>", "link text"),
    ("<a href=#>link<br>text</a>", "link\ntext"),
    ("<a href=#>link&amp;text</a>", "link&text"),
    ("<a href=#>LINK TEXT</a>", "LINK TEXT"),
    ("<a href=# style='text-transform: uppercase'>link text</a>", "LINK TEXT"),
])
def test_find_elements_link_text(session, inline, document, value):
    session.url = inline("<div><a href=#>not wanted</a><br/>{0}</div>".format(document))
    element = session.find.css("div", all=False)
    expected = session.execute_script("return document.links[1];")

    response = find_elements(session, element.id, "link text", value)
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1

    found_element = value[0]
    assert_same_element(session, found_element, expected)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>partial link text</a>", "link"),
    ("<a href=#>&nbsp;partial link text&nbsp;</a>", "link"),
    ("<a href=#>partial link text</a>", "k t"),
    ("<a href=#>partial link<br>text</a>", "k\nt"),
    ("<a href=#>partial link&amp;text</a>", "k&t"),
    ("<a href=#>PARTIAL LINK TEXT</a>", "LINK"),
    ("<a href=# style='text-transform: uppercase'>partial link text</a>", "LINK"),
])
def test_find_elements_partial_link_text(session, inline, document, value):
    session.url = inline("<div><a href=#>not wanted</a><br/>{0}</div>".format(document))
    element = session.find.css("div", all=False)
    expected = session.execute_script("return document.links[1];")

    response = find_elements(session, element.id, "partial link text", value)
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1

    found_element = value[0]
    assert_same_element(session, found_element, expected)


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//*[name()='a']")])
def test_xhtml_namespace(session, inline, using, value):
    session.url = inline("""<p><a href="#" id="linkText">full link text</a></p>""",
                         doctype="xhtml")
    from_element = session.execute_script("""return document.querySelector("p")""")
    expected = session.execute_script("return document.links[0]")

    response = find_elements(session, from_element.id, using, value)
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1

    found_element = value[0]
    assert_same_element(session, found_element, expected)


def test_parent_htmldocument(session, inline):
    session.url = inline("")
    from_element = session.execute_script("""return document.querySelector("body")""")
    expected = session.execute_script("return document.documentElement")

    response = find_elements(session, from_element.id, "xpath", "..")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1

    found_element = value[0]
    assert_same_element(session, found_element, expected)


def test_parent_of_document_node_errors(session, inline):
    session.url = inline("")
    from_element = session.execute_script("return document.documentElement")

    response = find_elements(session, from_element.id, "xpath", "..")
    assert_error(response, "invalid selector")
