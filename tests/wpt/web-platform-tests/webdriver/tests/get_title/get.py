from six import text_type

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def get_title(session):
    return session.transport.send(
        "GET", "session/{session_id}/title".format(**vars(session)))


def test_payload(session):
    session.start()

    response = get_title(session)
    value = assert_success(response)
    assert isinstance(value, text_type)


def test_no_top_browsing_context(session, closed_window):
    response = get_title(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    session.url = inline("<title>Foo</title>")

    response = get_title(session)
    assert_success(response, "Foo")


def test_with_duplicated_title(session):
    session.url = inline("<title>First</title><title>Second</title>")

    result = get_title(session)
    assert_success(result, "First")


def test_without_title(session):
    session.url = inline("<h2>Hello</h2>")

    result = get_title(session)
    assert_success(result, "")


def test_after_modification(session):
    session.url = inline("<title>Initial</title><h2>Hello</h2>")
    session.execute_script("document.title = 'Updated'")

    result = get_title(session)
    assert_success(result, "Updated")


def test_strip_and_collapse(session):
    document = "<title>   a b\tc\nd\t \n e\t\n </title><h2>Hello</h2>"
    session.url = inline(document)

    result = get_title(session)
    assert_success(result, "a b c d e")
