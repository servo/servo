import pytest
from webdriver import error

from tests.support.asserts import assert_success


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
