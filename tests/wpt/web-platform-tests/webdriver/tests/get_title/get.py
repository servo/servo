from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline
from tests.support.sync import Poll


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def get_title(session):
    return session.transport.send(
        "GET", "session/{session_id}/title".format(**vars(session)))


def test_no_browsing_context(session, closed_window):
    response = get_title(session)
    assert_error(response, "no such window")


def test_title_from_top_context(session):
    session.url = inline("<title>Foobar</title><h2>Hello</h2>")

    result = get_title(session)
    assert_success(result, read_global(session, "document.title"))


def test_title_with_duplicate_element(session):
    session.url = inline("<title>First</title><title>Second</title>")

    result = get_title(session)
    assert_success(result, read_global(session, "document.title"))


def test_title_without_element(session):
    session.url = inline("<h2>Hello</h2>")

    result = get_title(session)
    assert_success(result, read_global(session, "document.title"))


def test_title_after_modification(session):
    def title():
        return read_global(session, "document.title")

    session.url = inline("<title>Initial</title><h2>Hello</h2>")
    session.execute_script("document.title = 'Updated'")

    wait = Poll(session, message='Document title does not match "{}"'.format(title()))
    wait.until(lambda s: assert_success(get_title(s)) == title())


def test_title_strip_and_collapse(session):
    document = "<title>   a b\tc\nd\t \n e\t\n </title><h2>Hello</h2>"
    session.url = inline(document)

    result = get_title(session)
    assert_success(result, read_global(session, "document.title"))


def test_title_from_frame(session, create_frame):
    session.url = inline("<title>Parent</title>parent")

    session.switch_frame(create_frame())
    session.switch_frame(create_frame())

    result = get_title(session)
    assert_success(result, "Parent")
