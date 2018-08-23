import pytest
from webdriver import error

from tests.support.asserts import assert_success
from tests.support.inline import inline


def delete_session(session):
    return session.transport.send("DELETE", "session/{session_id}".format(**vars(session)))


def test_null_response_value(session):
    response = delete_session(session)
    value = assert_success(response)
    assert value is None

    # Need an explicit call to session.end() to notify the test harness
    # that a new session needs to be created for subsequent tests.
    session.end()


def test_dismissed_beforeunload_prompt(session):
    session.url = inline("""
      <input type="text">
      <script>
        window.addEventListener("beforeunload", function (event) {
          event.preventDefault();
        });
      </script>
    """)

    session.find.css("input", all=False).send_keys("foo")

    response = delete_session(session)
    assert_success(response)

    # A beforeunload prompt has to be automatically dismissed, and the session deleted
    with pytest.raises(error.InvalidSessionIdException):
        session.alert.text

    # Need an explicit call to session.end() to notify the test harness
    # that a new session needs to be created for subsequent tests.
    session.end()
