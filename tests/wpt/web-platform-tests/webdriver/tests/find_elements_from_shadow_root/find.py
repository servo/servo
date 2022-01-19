import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success


def find_elements(session, shadow_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/shadow/{shadow_id}/elements".format(
            session_id=session.session_id,
            shadow_id=shadow_id),
        {"using": using, "value": value})


def test_null_parameter_value(session, http, get_shadow_page):
    session.url = get_shadow_page("<div><a href=# id=linkText>full link text</a></div>")
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root

    path = "/session/{session_id}/shadow/{shadow_id}/elements".format(
        session_id=session.session_id, shadow_id=shadow_root.id)
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = find_elements(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = find_elements(session, "notReal", "css selector", "foo")
    assert_error(response, "no such window")


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


def test_detached_shadow_root(session, get_shadow_page):
    session.url = get_shadow_page("<div><input type='checkbox'/></div>")
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    session.refresh()

    response = find_elements(session, shadow_root.id, "css selector", "input")
    assert_error(response, "detached shadow root")


def test_find_elements_equivalence(session, get_shadow_page):
    session.url = get_shadow_page("<div><input id='check' type='checkbox'/><input id='text'/></div>")
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    response = find_elements(session, shadow_root.id, "css selector", "input")
    assert_success(response)


@pytest.mark.parametrize("using,value",
                         [("css selector", "#linkText"),
                          ("link text", "full link text"),
                          ("partial link text", "link text"),
                          ("tag name", "a"),
                          ("xpath", "//a")])
def test_find_elements(session, get_shadow_page, using, value):
    # Step 8 - 9
    session.url = get_shadow_page("<div><a href=# id=linkText>full link text</a></div>")
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    response = find_elements(session, shadow_root.id, using, value)
    assert_success(response)


@pytest.mark.parametrize("document,value", [
    ("<a href=#>link text</a>", "link text"),
    ("<a href=#>&nbsp;link text&nbsp;</a>", "link text"),
    ("<a href=#>link<br>text</a>", "link\ntext"),
    ("<a href=#>link&amp;text</a>", "link&text"),
    ("<a href=#>LINK TEXT</a>", "LINK TEXT"),
    ("<a href=# style='text-transform: uppercase'>link text</a>", "LINK TEXT"),
])
def test_find_elements_link_text(session, get_shadow_page, document, value):
    # Step 8 - 9
    session.url = get_shadow_page("<div><a href=#>not wanted</a><br/>{0}</div>".format(document))
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    expected = session.execute_script("return arguments[0].shadowRoot.querySelectorAll('a')[1]",
                                      args=(custom_element,))

    response = find_elements(session, shadow_root.id, "link text", value)
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
def test_find_elements_partial_link_text(session, get_shadow_page, document, value):
    # Step 8 - 9
    session.url = get_shadow_page("<div><a href=#>not wanted</a><br/>{0}</div>".format(document))
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    expected = session.execute_script("return arguments[0].shadowRoot.querySelectorAll('a')[1]",
                                      args=(custom_element,))

    response = find_elements(session, shadow_root.id, "partial link text", value)
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 1

    found_element = value[0]
    assert_same_element(session, found_element, expected)


@pytest.mark.parametrize("using,value", [("css selector", "#wontExist")])
def test_no_element(session, get_shadow_page, using, value):
    # Step 8 - 9
    session.url = get_shadow_page("<div></div>")
    custom_element = session.find.css("custom-shadow-element", all=False)
    shadow_root = custom_element.shadow_root
    response = find_elements(session, shadow_root.id, using, value)
    assert response.body["value"] == []
