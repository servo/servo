import json

from tests.support.asserts import assert_success


def get_status(session):
    return session.transport.send("GET", "/status")


def test_get_status_no_session(http):
    with http.get("/status") as response:
        # GET /status should never return an error
        assert response.status == 200

        # parse JSON response and unwrap 'value' property
        parsed_obj = json.loads(response.read().decode('utf-8'))
        value = parsed_obj["value"]

        # Let body be a new JSON Object with the following properties:
        # "ready"
        #       The remote end's readiness state.
        assert value["ready"] in [True, False]
        # "message"
        #       An implementation-defined string explaining the remote end's
        #       readiness state.
        assert isinstance(value["message"], basestring)


def test_status_with_session_running_on_endpoint_node(new_session, add_browser_capabilites):
    # For an endpoint node, the maximum number of active
    # sessions is 1: https://www.w3.org/TR/webdriver/#dfn-maximum-active-sessions
    # A session is open, so we expect `ready` to be False
    # 8.3 step 1.

    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})

    response = get_status(session)
    value = assert_success(response)
    assert value["ready"] is False
    assert "message" in value

    session.end()

    # Active session count is 0, meaning that the
    # readiness state of the server should be True
    # 8.3 step 1. Again
    response = get_status(session)
    value = assert_success(response)
    assert value["ready"] is True
    assert "message" in value
