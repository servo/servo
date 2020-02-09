from tests.support import platform_name
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/url".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session):
    response = navigate_to(session, inline("<div/>"))
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = navigate_to(session, "foo")
    assert_error(response, "no such window")


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
