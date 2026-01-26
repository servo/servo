import pytest

from tests.support.asserts import assert_error, assert_success
from . import query_permissions, set_permissions


# > 1. Let parameters be the parameters argument, converted to an IDL value of
# >    type PermissionSetParameters. If this throws an exception, return a
# >    WebDriver error with WebDriver error code invalid argument.
@pytest.mark.parametrize(
    "parameters",
    [
        # { "descriptor": { "name": "geolocation" }, "state": "granted" }
        {"descriptor": {"name": 23}, "state": "granted"},
        {"descriptor": {}, "state": "granted"},
        {"descriptor": {"name": "geolocation"}, "state": "Granted"},
        {"descriptor": 23, "state": "granted"},
        {"descriptor": "geolocation", "state": "granted"},
        {"descriptor": [{"name": "geolocation"}], "state": "granted"},
        [{"descriptor": {"name": "geolocation"}, "state": "granted"}],
    ],
)
def test_invalid_parameters(session, url, parameters):
    session.url = url("/common/blank.html", protocol="https")

    response = set_permissions(session, parameters)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("state", ["granted", "denied", "prompt"])
def test_set_to_state(session, url, state):
    session.url = url("/common/blank.html", protocol="https")
    parameters = {"descriptor": {"name": "geolocation"}, "state": state}

    response = set_permissions(session, parameters)
    try:
        assert_success(response)
    except AssertionError:
        # > 4. If parameters.state is an inappropriate permission state for any
        # >    implementation-defined reason, return a WebDriver error with
        # >    WebDriver error code invalid argument.
        assert_error(response, "invalid argument")
        return

    assert response.body.get("value") is None

    result = query_permissions(session, "geolocation")

    assert isinstance(result, dict)
    assert result.get("status") == "success"
    assert result.get("value") == state
