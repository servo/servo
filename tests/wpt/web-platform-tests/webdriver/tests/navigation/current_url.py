import json
import pytest
import types

from tests.support.inline import inline
from tests.support.asserts import assert_error, assert_success
from tests.support.wait import wait

alert_doc = inline("<script>window.alert()</script>")
frame_doc = inline("<p>frame")
one_frame_doc = inline("<iframe src='%s'></iframe>" % frame_doc)
two_frames_doc = inline("<iframe src='%s'></iframe>" % one_frame_doc)


# TODO(ato): 7.1 Get


def test_get_current_url_no_browsing_context(session, create_window):
    # 7.2 step 1
    session.window_handle = create_window()
    session.close()

    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert_error(result, "no such window")


def test_get_current_url_alert_prompt(session):
    # 7.2 step 2
    session.url = alert_doc

    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert_error(result, "unexpected alert open")

def test_get_current_url_matches_location(session):
    # 7.2 step 3
    url = session.execute_script("return window.location.href")
    assert session.url == url

def test_get_current_url_payload(session):
    # 7.2 step 4-5
    session.start()

    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert result.status == 200
    assert isinstance(result.body["value"], basestring)

def test_get_current_url_special_pages(session):
    session.url = "about:blank"

    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert_success(result, "about:blank")

# TODO(ato): This test requires modification to pass on Windows
def test_get_current_url_file_protocol(session):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    session.url = "file:///"

    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert_success(result, "file:///")

# TODO(ato): Test for http:// and https:// protocols.
# We need to expose a fixture for accessing
# documents served by wptserve in order to test this.

def test_set_malformed_url(session):
    result = session.transport.send("POST",
                                    "session/%s/url" % session.session_id,
                                    {"url": "foo"})

    assert_error(result, "invalid argument")

def test_get_current_url_after_modified_location(session):
    start = session.transport.send("GET", "session/%s/url" % session.session_id)
    session.execute_script("window.location.href = 'about:blank#wd_test_modification'")
    wait(session,
         lambda s: s.transport.send("GET", "session/%s/url" % session.session_id) != start.body["value"],
         "URL did not change")
    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert_success(result, "about:blank#wd_test_modification")

def test_get_current_url_nested_browsing_context(session, create_frame):
    session.url = "about:blank#wd_from_within_frame"
    session.switch_frame(create_frame())

    result = session.transport.send("GET", "session/%s/url" % session.session_id)

    assert_success(result, "about:blank#wd_from_within_frame")

def test_get_current_url_nested_browsing_contexts(session):
    session.url = two_frames_doc
    top_level_url = session.url

    outer_frame = session.find.css("iframe", all=False)
    session.switch_frame(outer_frame)

    inner_frame = session.find.css("iframe", all=False)
    session.switch_frame(inner_frame)

    assert session.url == top_level_url
