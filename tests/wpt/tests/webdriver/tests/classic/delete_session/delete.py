import pytest
from webdriver import error
from webdriver.transport import HTTPWireProtocol

from tests.support.classic.asserts import assert_success


def delete_session(session):
    return session.transport.send("DELETE", "session/{session_id}".format(**vars(session)))


def test_null_response_value(session):
    response = delete_session(session)
    value = assert_success(response)
    assert value is None

    # Need an explicit call to session.end() to notify the test harness
    # that a new session needs to be created for subsequent tests.
    session.end()


def test_accepted_beforeunload_prompt(session, url):
    session.url = url("/webdriver/tests/support/html/beforeunload.html")

    session.find.css("input", all=False).send_keys("foo")

    response = delete_session(session)
    assert_success(response)

    # A beforeunload prompt has to be automatically accepted, and the session deleted
    with pytest.raises(error.InvalidSessionIdException):
        session.alert.text

    # Need an explicit call to session.end() to notify the test harness
    # that a new session needs to be created for subsequent tests.
    session.end()


def test_delete_session_with_different_connection(session, configuration):
    # The session ID should remain valid on any HTTP connection to the
    # remote end, not just the one that created it. Close the original
    # connection first to prove it doesn't need to stay open.
    session.transport.close()

    other_transport = HTTPWireProtocol(configuration["host"], configuration["port"])

    response = other_transport.send("DELETE", f"session/{session.session_id}")
    value = assert_success(response)
    assert value is None

    # Need an explicit call to session.end() to notify the test harness
    # that a new session needs to be created for subsequent tests.
    session.end()
