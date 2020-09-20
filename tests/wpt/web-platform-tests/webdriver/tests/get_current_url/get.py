from six import text_type

from tests.support import platform_name
from tests.support.inline import inline
from tests.support.asserts import assert_error, assert_success

doc = inline("<p>frame")
alert_doc = inline("<script>window.alert()</script>")


def get_current_url(session):
    return session.transport.send(
        "GET", "session/{session_id}/url".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = get_current_url(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    session.url = doc

    response = get_current_url(session)
    assert_success(response, doc)


def test_get_current_url_matches_location(session):
    session.url = doc

    response = get_current_url(session)
    assert_success(response, doc)


def test_get_current_url_payload(session):
    session.start()

    response = get_current_url(session)
    value = assert_success(response)
    assert isinstance(value, text_type)


def test_get_current_url_special_pages(session):
    session.url = "about:blank"

    response = get_current_url(session)
    assert_success(response, "about:blank")


def test_get_current_url_file_protocol(session, server_config):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    path = server_config["doc_root"]
    if platform_name == "windows":
        # Convert the path into the format eg. /c:/foo/bar
        path = "/{}".format(path.replace("\\", "/"))
    url = u"file://{}".format(path)
    session.url = url

    response = get_current_url(session)
    if response.status == 200 and response.body['value'].endswith('/'):
        url += '/'
    assert_success(response, url)


# TODO(ato): Test for http:// and https:// protocols.
# We need to expose a fixture for accessing
# documents served by wptserve in order to test this.


def test_set_malformed_url(session):
    response = session.transport.send(
        "POST",
        "session/%s/url" % session.session_id, {"url": "foo"})

    assert_error(response, "invalid argument")


def test_get_current_url_after_modified_location(session):
    session.url = doc

    response = get_current_url(session)
    assert_success(response, doc)

    hash_doc = "{}#foo".format(doc)
    session.url = hash_doc

    response = get_current_url(session)
    assert_success(response, hash_doc)
