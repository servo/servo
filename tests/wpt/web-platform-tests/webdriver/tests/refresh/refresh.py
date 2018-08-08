import pytest

from webdriver.error import NoSuchElementException, StaleElementReferenceException

from tests.support.inline import inline
from tests.support.asserts import assert_error, assert_success


def refresh(session):
    return session.transport.send(
        "POST", "session/{session_id}/refresh".format(**vars(session)))


def test_null_response_value(session):
    session.url = inline("<div>")

    response = refresh(session)
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = refresh(session)
    assert_error(response, "no such window")


def test_basic(session):
    url = inline("<div id=foo>")

    session.url = url
    element = session.find.css("#foo", all=False)

    response = refresh(session)
    assert_success(response)

    with pytest.raises(StaleElementReferenceException):
        element.property("id")

    assert session.url == url
    assert session.find.css("#foo", all=False)


def test_dismissed_beforeunload(session):
    url_beforeunload = inline("""
      <input type="text">
      <script>
        window.addEventListener("beforeunload", function (event) {
          event.preventDefault();
        });
      </script>
    """)

    session.url = url_beforeunload
    element = session.find.css("input", all=False)
    element.send_keys("bar")

    response = refresh(session)
    assert_success(response)

    with pytest.raises(StaleElementReferenceException):
        element.property("id")

    session.find.css("input", all=False)


def test_history_pushstate(session, url):
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

    with pytest.raises(StaleElementReferenceException):
        element.property("id")


def test_refresh_switches_to_parent_browsing_context(session, create_frame):
    session.url = inline("<div id=foo>")

    session.switch_frame(create_frame())
    with pytest.raises(NoSuchElementException):
        session.find.css("#foo", all=False)

    response = refresh(session)
    assert_success(response)

    session.find.css("#foo", all=False)
