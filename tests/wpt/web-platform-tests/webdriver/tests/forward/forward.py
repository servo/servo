from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def forward(session):
    return session.transport.send(
        "POST", "session/{session_id}/forward".format(**vars(session)))


def test_null_response_value(session):
    session.url = inline("<div>")
    session.url = inline("<p>")
    session.back()

    response = forward(session)
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = forward(session)
    assert_error(response, "no such window")


def test_no_browsing_history(session):
    url = inline("<div id=foo>")

    session.url = url
    element = session.find.css("#foo", all=False)

    response = forward(session)
    assert_success(response)

    assert session.url == url
    assert element.property("id") == "foo"


def test_data_urls(session):
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

    session.back()
    assert session.url == pushstate_page
    assert session.execute_script("return history.state;") is None

    response = forward(session)
    assert_success(response)

    assert session.url == "{}#pushstate".format(pushstate_page)
    assert session.execute_script("return history.state;") == {"foo": "bar"}
