import pytest
import json


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
