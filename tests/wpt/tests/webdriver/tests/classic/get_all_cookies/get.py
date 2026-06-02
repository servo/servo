import pytest

from tests.support.asserts import assert_error, assert_success
from . import get_all_cookies


@pytest.fixture(autouse=True)
def clean_up_cookies(session):
    # Ensure that any test in the file does not navigate away once done with checking the cookies.
    session.transport.send("DELETE", "session/%s/cookie" % session.session_id)


def test_no_top_browsing_context(session, closed_window):
    response = get_all_cookies(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_all_cookies(session)
    assert_error(response, "no such window")


def test_get_multiple_cookies(session, url):
    session.url = url("/common/blank.html")
    session.execute_script("document.cookie = 'foo=bar'")
    session.execute_script("document.cookie = 'hello=world'")

    result = get_all_cookies(session)
    cookies = assert_success(result)
    assert isinstance(cookies, list)

    expected_cookies = [
        {
            "name": "foo",
            "value": "bar",
            "path": "/common",
            "domain": "web-platform" + ".test",
            "secure": False,
            "httpOnly": False,
            "sameSite": "None",
        },
        {
            "name": "hello",
            "value": "world",
            "path": "/common",
            "domain": "web-platform" + ".test",
            "secure": False,
            "httpOnly": False,
            "sameSite": "None",
        },
    ]

    assert len(cookies) == len(expected_cookies)


def test_get_cookies_only_from_active_document(session, url, create_cookie):
    # Set cookies for two different pages.
    create_cookie("foo", value="bar", path="/common/blank.html")
    create_cookie("hello", value="world", path="/common/blank.html")
    create_cookie("abc", value="xyz", path="/example/index.html")
    create_cookie("mock", value="dummy", path="/example/index.html")

    session.url = url("/common/blank.html")
    result = get_all_cookies(session)
    cookies = assert_success(result)
    assert isinstance(cookies, list)

    expected_cookies = [
        {
            "name": "foo",
            "value": "bar",
            "path": "/common/blank.html",
            "domain": "web-platform" + ".test",
            "secure": False,
            "httpOnly": False,
            "sameSite": "None",
        },
        {
            "name": "hello",
            "value": "world",
            "path": "/common/blank.html",
            "domain": "web-platform" + ".test",
            "secure": False,
            "httpOnly": False,
            "sameSite": "None",
        },
    ]

    assert cookies == expected_cookies
