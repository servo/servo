from tests.support.asserts import assert_error, assert_success


def back(session):
    return session.transport.send(
        "POST", "session/{session_id}/back".format(**vars(session)))


def test_null_response_value(session, inline):
    session.url = inline("<div>")
    session.url = inline("<p>")

    response = back(session)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    response = back(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = back(session)
    assert_success(response)


def test_no_browsing_history(session):
    response = back(session)
    assert_success(response)


def test_data_urls(session, inline):
    test_pages = [
        inline("<p id=1>"),
        inline("<p id=2>"),
    ]

    for page in test_pages:
        session.url = page
    assert session.url == test_pages[1]

    response = back(session)
    assert_success(response)
    assert session.url == test_pages[0]


def test_dismissed_beforeunload(session, inline):
    url_beforeunload = inline("""
      <input type="text">
      <script>
        window.addEventListener("beforeunload", function (event) {
          event.preventDefault();
        });
      </script>
    """)

    session.url = inline("<div id=foo>")
    session.url = url_beforeunload

    element = session.find.css("input", all=False)
    element.send_keys("bar")

    response = back(session)
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
    assert session.url == test_pages[2]

    response = back(session)
    assert_success(response)
    assert session.url == test_pages[1]

    response = back(session)
    assert_success(response)
    assert session.url == test_pages[0]


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

    response = back(session)
    assert_success(response)

    assert session.url == pushstate_page
    assert session.execute_script("return history.state;") is None


def test_removed_iframe(session, url, inline):
    page = inline("<p>foo")

    session.url = page
    session.url = url("/webdriver/tests/support/html/frames_no_bfcache.html")

    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    response = back(session)
    assert_success(response)

    assert session.url == page
