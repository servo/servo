from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def get_page_source(session):
    return session.transport.send(
        "GET", "session/{session_id}/source".format(**vars(session)))


def test_no_browsing_context(session, closed_window):
    response = get_page_source(session)
    assert_error(response, "no such window")


def test_source_matches_outer_html(session):
    session.url = inline("<html><head><title>Cheese</title><body>Peas")

    expected = session.execute_script("return document.documentElement.outerHTML")

    response = get_page_source(session)
    assert_success(response, expected)
