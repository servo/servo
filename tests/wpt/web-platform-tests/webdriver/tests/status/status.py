import json

from six import text_type

from tests.support.asserts import assert_success


def get_status(session):
    return session.transport.send("GET", "/status")


def test_get_status_no_session(http):
    with http.get("/status") as response:
        # GET /status should never return an error
        assert response.status == 200

        parsed_obj = json.loads(response.read().decode("utf-8"))
        value = parsed_obj["value"]

        assert value["ready"] in [True, False]
        assert isinstance(value["message"], text_type)


def test_status_with_session_running_on_endpoint_node(session):
    response = get_status(session)
    value = assert_success(response)
    assert value["ready"] is False
    assert "message" in value

    session.end()

    response = get_status(session)
    value = assert_success(response)
    assert value["ready"] is True
    assert "message" in value
