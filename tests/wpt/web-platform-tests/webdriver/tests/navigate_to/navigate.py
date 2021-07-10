import time

import pytest

from tests.support import platform_name
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/url".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, inline):
    response = navigate_to(session, inline("<div/>"))
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    response = navigate_to(session, "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame, inline):
    doc = inline("<p>foo")

    response = navigate_to(session, doc)
    assert_success(response)

    assert session.url == doc


def test_file_protocol(session, server_config):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    path = server_config["doc_root"]
    if platform_name == "windows":
        # Convert the path into the format eg. /c:/foo/bar
        path = "/{}".format(path.replace("\\", "/"))
    url = u"file://{}".format(path)

    response = navigate_to(session, url)
    assert_success(response)

    if session.url.endswith('/'):
        url += '/'
    assert session.url == url


@pytest.mark.capabilities({"pageLoadStrategy": "eager"})
def test_utf8_meta_tag_after_1024_bytes(session, url):
    page = url("/webdriver/tests/support/html/meta-utf8-after-1024-bytes.html")

    # Loading the page will cause a real parse commencing, and a renavigation
    # to the same URL getting triggered subsequently. Test that the navigate
    # command waits long enough.
    response = navigate_to(session, page)
    assert_success(response)

    # If the command returns too early the property will be reset due to the
    # subsequent page load.
    session.execute_script("window.foo = 'bar'")

    # Use delay to allow a possible missing subsequent navigation to start
    time.sleep(1)

    assert session.execute_script("return window.foo") == "bar"
