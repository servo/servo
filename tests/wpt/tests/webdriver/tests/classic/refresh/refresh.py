import pytest

from webdriver import error

from tests.support.asserts import assert_error, assert_success


def refresh(session):
    return session.transport.send(
        "POST", "session/{session_id}/refresh".format(**vars(session)))


def test_null_response_value(session, inline):
    session.url = inline("<div>")

    response = refresh(session)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    response = refresh(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame, inline):
    url = inline("<div id=foo>")

    session.url = url
    element = session.find.css("#foo", all=False)

    response = refresh(session)
    assert_success(response)

    with pytest.raises(error.StaleElementReferenceException):
        element.property("id")

    assert session.url == url
    assert session.find.css("#foo", all=False)


@pytest.mark.parametrize("protocol,parameters", [
    ("http", ""),
    ("https", ""),
    ("https", {"pipe": "header(Cross-Origin-Opener-Policy,same-origin)"})
], ids=["http", "https", "https coop"])
def test_seen_nodes(session, get_test_page, protocol, parameters):
    page = get_test_page(parameters=parameters, protocol=protocol)

    session.url = page

    element = session.find.css("#custom-element", all=False)
    shadow_root = element.shadow_root

    response = refresh(session)
    assert_success(response)

    with pytest.raises(error.StaleElementReferenceException):
        element.name
    with pytest.raises(error.DetachedShadowRootException):
        shadow_root.find_element("css selector", "in-shadow-dom")

    session.find.css("#custom-element", all=False)


def test_history_pushstate(session, inline):
    pushstate_page = inline("""
      <script>
        function pushState() {
          history.pushState({foo: "bar"}, "", "#pushstate");
        }
      </script>
      <a onclick="javascript:pushState();">click</a>
    """)

    session.url = pushstate_page

    session.find.css("a", all=False).click()
    assert session.url == "{}#pushstate".format(pushstate_page)
    assert session.execute_script("return history.state;") == {"foo": "bar"}

    session.execute_script("""
      let elem = window.document.createElement('div');
      window.document.body.appendChild(elem);
    """)
    element = session.find.css("div", all=False)

    response = refresh(session)
    assert_success(response)

    assert session.url == "{}#pushstate".format(pushstate_page)
    assert session.execute_script("return history.state;") == {"foo": "bar"}

    with pytest.raises(error.StaleElementReferenceException):
        element.property("id")


def test_refresh_switches_to_parent_browsing_context(session, create_frame, inline):
    session.url = inline("<div id=foo>")

    session.switch_frame(create_frame())
    with pytest.raises(error.NoSuchElementException):
        session.find.css("#foo", all=False)

    response = refresh(session)
    assert_success(response)

    session.find.css("#foo", all=False)
