import pytest

from tests.support.asserts import assert_error, assert_same_element, assert_success
from tests.support.inline import inline


def find_elements(session, element, using, value):
    return session.transport.send("POST",
                                  "session/%s/element/%s/elements" % (session.session_id, element),
                                  {"using": using, "value": value})

@pytest.mark.parametrize("using", [("a"), (True), (None), (1), ([]), ({})])
def test_invalid_using_argument(session, using):
    # Step 1 - 2
    response = find_elements(session, "notReal", using, "value")
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, [], {}])
def test_invalid_selector_argument(session, value):
    # Step 3 - 4
    response = find_elements(session, "notReal", "css selector", value)
    assert_error(response, "invalid argument")


def test_closed_context(session, create_window):
    # Step 5
    new_window = create_window()
    session.window_handle = new_window
    session.close()

    response = find_elements(session, "notReal", "css selector", "foo")

    assert_error(response, "no such window")


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
def test_find_elements(session, using, value):
    # Step 8 - 9
    session.url = inline("<div><a href=# id=linkText>full link text</a></div>")
    element = session.find.css("div", all=False)
    response = find_elements(session, element.id, using, value)
    assert_success(response)


@pytest.mark.parametrize("using,value", [("css selector", "#wontExist")])
def test_no_element(session, using, value):
    # Step 8 - 9
    session.url = inline("<div></div>")
    element = session.find.css("div", all=False)
    response = find_elements(session, element.id, using, value)
    assert response.body["value"] == []


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//*[name()='a']")])
def test_xhtml_namespace(session, using, value):
    session.url = inline("""<p><a href="#" id="linkText">full link text</a></p>""", doctype="xhtml")
    from_element = session.execute_script("""return document.querySelector("p")""")
    expected = session.execute_script("return document.links[0]")

    response = find_elements(session, from_element.id, using, value)
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1

    found_element = value[0]
    assert_same_element(session, found_element, expected)


def test_parent_htmldocument(session):
    session.url = inline("")
    from_element = session.execute_script("return document.documentElement")

    response = find_elements(session, from_element.id, "xpath", "..")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1
