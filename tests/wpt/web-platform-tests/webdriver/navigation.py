import contextlib
import httplib
import json
import pytest
import types
import urllib

import webdriver


def inline(doc):
    return "data:text/html;charset=utf-8,%s" % urllib.quote(doc)


alert_doc = inline("<script>window.alert()</script>")
frame_doc = inline("<p>frame")
one_frame_doc = inline("<iframe src='%s'></iframe>" % frame_doc)
two_frames_doc = inline("<iframe src='%s'></iframe>" % one_frame_doc)


class HTTPRequest(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port

    def head(self, path):
        return self._request("HEAD", path)

    def get(self, path):
        return self._request("GET", path)

    @contextlib.contextmanager
    def _request(self, method, path):
        conn = httplib.HTTPConnection(self.host, self.port)
        try:
            conn.request(method, path)
            yield conn.getresponse()
        finally:
            conn.close()


@pytest.fixture(scope="function")
def http(request, session):
    return HTTPRequest(session.transport.host, session.transport.port)


@pytest.fixture
def new_window(session):
    """Open new window and return the window handle."""
    windows_before = session.window_handles
    name = session.execute_script("window.open()")
    assert len(session.window_handles) == len(windows_before) + 1
    new_windows = session.window_handles - windows_before
    return new_windows.pop()


# TODO(ato): 7.1 Get


def test_get_current_url_no_browsing_context(session, new_window):
    # 7.2 step 1
    session.window_handle = new_window
    session.close()
    with pytest.raises(webdriver.NoSuchWindowException):
        session.url = "about:blank"


def test_get_current_url_alert_prompt(session):
    # 7.2 step 2
    import time
    session.url = alert_doc
    with pytest.raises(webdriver.UnexpectedAlertOpenException):
        session.url = "about:blank"


def test_get_current_url_matches_location(session):
    # 7.2 step 3
    url = session.execute_script("return window.location.href")
    assert session.url == url


def test_get_current_url_payload(http, session):
    # 7.2 step 4-5
    session.start()
    with http.get("/session/%s/url" % session.session_id) as resp:
        assert resp.status == 200
        body = json.load(resp)
    assert "value" in body
    assert isinstance(body["value"], types.StringTypes)


def test_get_current_url_special_pages(session):
    session.url = "about:blank"
    assert session.url == "about:blank"


# TODO(ato): This test requires modification to pass on Windows
def test_get_current_url_file_protocol(session):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    session.url = "file:///"
    assert session.url == "file:///"


# TODO(ato): Test for http:// and https:// protocols.
# We need to expose a fixture for accessing
# documents served by wptserve in order to test this.


def test_get_current_url_malformed_url(session):
    session.url = "foo"
    assert session.url


def test_get_current_url_after_modified_location(session):
    session.execute_script("window.location.href = 'about:blank'")
    assert session.url == "about:blank"


def test_get_current_url_nested_browsing_context(session):
    session.url = one_frame_doc
    top_level_url = session.url
    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    assert session.url == top_level_url


def test_get_current_url_nested_browsing_contexts(session):
    session.url = two_frames_doc
    top_level_url = session.url

    outer_frame = session.find("iframe", all=False)
    session.switch_frame(outer_frame)

    inner_frame = session.find("iframe", all=False)
    session.switch_frame(frame)

    assert session.url == top_level_url
