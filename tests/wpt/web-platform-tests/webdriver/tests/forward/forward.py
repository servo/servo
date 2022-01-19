import pytest
from webdriver import error

from tests.support.asserts import assert_error, assert_success


def forward(session):
    return session.transport.send(
        "POST", "session/{session_id}/forward".format(**vars(session)))


def test_null_response_value(session, inline):
    session.url = inline("<div>")
    session.url = inline("<p>")
    session.back()

    response = forward(session)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    response = forward(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = forward(session)
    assert_success(response)


def test_no_browsing_history(session, inline):
    url = inline("<div id=foo>")

    session.url = url
    element = session.find.css("#foo", all=False)

    response = forward(session)
    assert_success(response)

    assert session.url == url
    assert element.property("id") == "foo"


def test_data_urls(session, inline):
    test_pages = [
        inline("<p id=1>"),
        inline("<p id=2>"),
    ]

    for page in test_pages:
        session.url = page

    session.back()
    assert session.url == test_pages[0]

    response = forward(session)
    assert_success(response)
    assert session.url == test_pages[1]


def test_dismissed_beforeunload(session, inline):
    url_beforeunload = inline("""
      <input type="text">
      <script>
        window.addEventListener("beforeunload", function (event) {
          event.preventDefault();
        });
      </script>
    """)

    session.url = url_beforeunload
    session.url = inline("<div id=foo>")
    session.back()

    element = session.find.css("input", all=False)
    element.send_keys("bar")

    response = forward(session)
    assert_success(response)

    assert session.url != url_beforeunload


def test_fragments(session, url):
    test_pages = [
        url("/common/blank.html"),
        url("/common/blank.html#1234"),
        url("/common/blank.html#5678"),
    ]

    for page in test_pages:
        session.url = page

    session.back()
    assert session.url == test_pages[1]

    session.back()
    assert session.url == test_pages[0]

    response = forward(session)
    assert_success(response)
    assert session.url == test_pages[1]

    response = forward(session)
    assert_success(response)
    assert session.url == test_pages[2]


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

    session.back()
    assert session.url == pushstate_page
    assert session.execute_script("return history.state;") is None

    response = forward(session)
    assert_success(response)

    assert session.url == "{}#pushstate".format(pushstate_page)
    assert session.execute_script("return history.state;") == {"foo": "bar"}


def test_removed_iframe(session, url, inline):
    page = inline("<p>foo")

    session.url = url("/webdriver/tests/support/html/frames_no_bfcache.html")
    session.url = page

    session.back()

    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    response = forward(session)
    assert_success(response)

    assert session.url == page


# Capability needed as long as no valid certificate is available:
#   https://github.com/web-platform-tests/wpt/issues/28847
@pytest.mark.capabilities({"acceptInsecureCerts": True})
def test_cross_origin(session, url):
    base_path = ("/webdriver/tests/support/html/subframe.html" +
                 "?pipe=header(Cross-Origin-Opener-Policy,same-origin")
    first_page = url(base_path, protocol="https")
    second_page = url(base_path, protocol="https", domain="alt")

    session.url = first_page
    session.url = second_page
    session.back()

    elem = session.find.css("#delete", all=False)

    response = forward(session)
    assert_success(response)

    assert session.url == second_page

    with pytest.raises(error.StaleElementReferenceException):
        elem.click()
    elem = session.find.css("#delete", all=False)
