import pytest

from tests.support import platform_name
from tests.support.asserts import assert_error, assert_success


@pytest.fixture
def doc(inline):
    return inline("<p>frame")


def get_current_url(session):
    return session.transport.send(
        "GET", "session/{session_id}/url".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = get_current_url(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame, doc):
    session.url = doc

    response = get_current_url(session)
    assert_success(response, doc)


def test_get_current_url_matches_location(session, doc):
    session.url = doc

    response = get_current_url(session)
    assert_success(response, doc)


def test_get_current_url_payload(session):
    session.start()

    response = get_current_url(session)
    value = assert_success(response)
    assert isinstance(value, str)


def test_get_current_url_special_pages(session):
    session.url = "about:blank"

    response = get_current_url(session)
    assert_success(response, "about:blank")


# TODO(ato): Test for http:// and https:// protocols.
# We need to expose a fixture for accessing
# documents served by wptserve in order to test this.


def test_set_malformed_url(session):
    response = session.transport.send(
        "POST",
        "session/%s/url" % session.session_id, {"url": "foo"})

    assert_error(response, "invalid argument")


def test_get_current_url_after_modified_location(session, doc):
    session.url = doc

    response = get_current_url(session)
    assert_success(response, doc)

    hash_doc = "{}#foo".format(doc)
    session.url = hash_doc

    response = get_current_url(session)
    assert_success(response, hash_doc)
