from tests.support.asserts import assert_error, assert_success
import pytest


def query(session, name):
    script = """
        var done = arguments[0];
        navigator.permissions.query({ name: '%s' })
          .then(function(value) {
              done({ status: 'success', value: value && value.state });
            }, function(error) {
              done({ status: 'error', value: error && error.message });
            });
    """ % name

    return session.transport.send(
        "POST", "/session/{session_id}/execute/async".format(**vars(session)),
        {
            "script": script,
            "args": []
        })


# > 1. Let parameters be the parameters argument, converted to an IDL value of
# >    type PermissionSetParameters. If this throws an exception, return a
# >    WebDriver error with WebDriver error code invalid argument.
@pytest.mark.parametrize(
    "parameters",
    [
        #{ "descriptor": { "name": "geolocation" }, "state": "granted" }
        {
            "descriptor": {
                "name": 23
            },
            "state": "granted"
        },
        {
            "descriptor": {},
            "state": "granted"
        },
        {
            "descriptor": {
                "name": "geolocation"
            },
            "state": "Granted"
        },
        {
            "descriptor": 23,
            "state": "granted"
        },
        {
            "descriptor": "geolocation",
            "state": "granted"
        },
        {
            "descriptor": [{
                "name": "geolocation"
            }],
            "state": "granted"
        },
        [{
            "descriptor": {
                "name": "geolocation"
            },
            "state": "granted"
        }],
    ])
def test_invalid_parameters(session, url, parameters):
    session.url = url("/common/blank.html", protocol="https")
    response = session.transport.send(
        "POST", "/session/{session_id}/permissions".format(**vars(session)),
        parameters)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("state", ["granted", "denied", "prompt"])
def test_set_to_state(session, url, state):
    session.url = url("/common/blank.html", protocol="https")
    parameters = {"descriptor": {"name": "geolocation"}, "state": state}
    response = session.transport.send(
        "POST", "/session/{session_id}/permissions".format(**vars(session)),
        parameters)

    try:
        assert_success(response)
    except AssertionError:
        # > 4. If parameters.state is an inappropriate permission state for any
        # >    implementation-defined reason, return a WebDriver error with
        # >    WebDriver error code invalid argument.
        assert_error(response, "invalid argument")
        return

    assert response.body.get("value") is None

    response = query(session, "geolocation")

    assert_success(response)
    result = response.body.get("value")

    assert isinstance(result, dict)
    assert result.get("status") == "success"
    assert result.get("value") == state
