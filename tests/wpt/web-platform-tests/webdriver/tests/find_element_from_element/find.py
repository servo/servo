import pytest

from tests.support.asserts import assert_error, assert_same_element, assert_success
from tests.support.inline import inline


def find_element(session, element_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/element".format(
            session_id=session.session_id,
            element_id=element_id),
        {"using": using, "value": value})


@pytest.mark.parametrize("using", ["a", True, None, 1, [], {}])
def test_invalid_using_argument(session, using):
    # Step 1 - 2
    response = find_element(session, "notReal", using, "value")
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, [], {}])
def test_invalid_selector_argument(session, value):
    # Step 3 - 4
    response = find_element(session, "notReal", "css selector", value)
    assert_error(response, "invalid argument")


def test_closed_context(session, create_window):
    # Step 5
    new_window = create_window()
    session.window_handle = new_window
    session.close()

    response = find_element(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
def test_find_element(session, using, value):
    # Step 8 - 9
    session.url = inline("<div><a href=# id=linkText>full link text</a></div>")
    element = session.find.css("div", all=False)
    response = find_element(session, element.id, using, value)
    assert_success(response)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>link text</a>", "link text"),
    ("<a href=#>&nbsp;link text&nbsp;</a>", "link text"),
    ("<a href=#>link<br>text</a>", "link\ntext"),
    ("<a href=#>link&amp;text</a>", "link&text"),
    ("<a href=#>LINK TEXT</a>", "LINK TEXT"),
    ("<a href=# style='text-transform: uppercase'>link text</a>", "LINK TEXT"),
])
def test_find_element_link_text(session, document, value):
    # Step 8 - 9
    session.url = inline("<div>{0}</div>".format(document))
    element = session.find.css("div", all=False)

    response = find_element(session, element.id, "link text", value)
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
def test_find_element_partial_link_text(session, document, value):
    # Step 8 - 9
    session.url = inline("<div>{0}</div>".format(document))
    element = session.find.css("div", all=False)

    response = find_element(session, element.id, "partial link text", value)
    assert_success(response)


@pytest.mark.parametrize("using,value",[("css selector", "#wontExist")])
def test_no_element(session, using, value):
    # Step 8 - 9
    session.url = inline("<div></div>")
    element = session.find.css("div", all=False)
    response = find_element(session, element.id, using, value)
    assert_error(response, "no such element")


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//*[name()='a']")])
def test_xhtml_namespace(session, using, value):
    session.url = inline("""<p><a href="#" id="linkText">full link text</a></p>""",
                         doctype="xhtml")
    from_element = session.execute_script("""return document.querySelector("p")""")
    expected = session.execute_script("return document.links[0]")

    response = find_element(session, from_element.id, using, value)
    value = assert_success(response)
    assert_same_element(session, value, expected)


def test_parent_htmldocument(session):
    session.url = inline("")
    from_element = session.execute_script("""return document.querySelector("body")""")
    expected = session.execute_script("return document.documentElement")

    response = find_element(session, from_element.id, "xpath", "..")
    value = assert_success(response)
    assert_same_element(session, value, expected)


def test_parent_of_document_node_errors(session):
    session.url = inline("")
    from_element = session.execute_script("return document.documentElement")

    response = find_element(session, from_element.id, "xpath", "..")
    assert_error(response, "invalid selector")
