import pytest
from webdriver import error

from tests.support.asserts import assert_success
from tests.support.inline import inline


def delete_session(session):
    return session.transport.send("DELETE", "session/{session_id}".format(**vars(session)))


def test_delete_session_with_dismissed_beforeunload_prompt(session):
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
    with pytest.raises(error.SessionNotCreatedException):
        session.alert.text
