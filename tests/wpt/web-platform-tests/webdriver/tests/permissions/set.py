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
        {"script": script, "args": []})

# > 1. Let parameters be the parameters argument, converted to an IDL value of
# >    type PermissionSetParameters. If this throws an exception, return a
# >    WebDriver error with WebDriver error code invalid argument.
@pytest.mark.parametrize("parameters", [
    #{ "descriptor": { "name": "geolocation" }, "state": "granted" }
    { "descriptor": { "name": 23 }, "state": "granted" },
    { "descriptor": { }, "state": "granted" },
    { "descriptor": { "name": "geolocation" }, "state": "Granted" },
    { "descriptor": 23, "state": "granted" },
    { "descriptor": "geolocation", "state": "granted" },
    { "descriptor": [ { "name": "geolocation" } ], "state": "granted" },
    [ { "descriptor": { "name": "geolocation" }, "state": "granted" } ],
    { "descriptor": { "name": "geolocation" }, "state": "granted", "oneRealm": 23 }
])
def test_invalid_parameters(session, parameters):
    response = session.transport.send(
        "POST",
        "/session/{session_id}/permissions".format(**vars(session)),
        parameters
    )
    assert_error(response, "invalid argument")

# > 6. If settings is a non-secure context and rootDesc.name isn't allowed in
# >    non-secure contexts, return a WebDriver error with WebDriver error code
# >    invalid argument.
@pytest.mark.parametrize("state", ["granted", "denied", "prompt"])
def test_non_secure_context(session, url, state):
    session.url = url("/common/blank.html", protocol="http")
    response = session.transport.send(
        "POST", "/session/{session_id}/permissions".format(**vars(session)),
        { "descriptor": { "name": "push" }, "state": state }
    )

    assert_error(response, "invalid argument")

@pytest.mark.parametrize("state", ["granted", "denied", "prompt"])
@pytest.mark.parametrize("realmSetting", [
    { "oneRealm": True },
    { "oneRealm": False },
    {}
])
def test_set_to_state(session, state, realmSetting):
    parameters = { "descriptor": { "name": "geolocation" }, "state": state }
    parameters.update(realmSetting)
    response = session.transport.send(
        "POST", "/session/{session_id}/permissions".format(**vars(session)),
        parameters
    )

    try:
        assert_success(response)
    except AssertionError:
        # > 4. If parameters.state is an inappropriate permission state for any
        # >    implementation-defined reason, return a WebDriver error with
        # >    WebDriver error code invalid argument.
        assert_error(response, "invalid argument")
        return

    assert response.body.get("value") == None

    response = query(session, "geolocation")

    assert_success(response)
    result = response.body.get("value")

    assert isinstance(result, dict)
    assert result.get("status") == "success"
    assert result.get("value") == state

# > 7. If parameters.oneRealm is true, [...]
# > 8. Otherwise, let targets be a list containing all environment settings
# >    objects whose origin is the same as the origin of settings.
#
# Ensure that all realms are affected when `oneRealm` is not enabled.
@pytest.mark.parametrize("state", ["granted", "denied", "prompt"])
@pytest.mark.parametrize("realmSetting", [
    { "oneRealm": False },
    {}
])
def test_set_to_state_cross_realm(session, create_window, state, realmSetting):
    original_window = session.window_handle
    session.window_handle = create_window()
    parameters = { "descriptor": { "name": "geolocation" }, "state": state }
    parameters.update(realmSetting)

    response = session.transport.send(
        "POST", "/session/{session_id}/permissions".format(**vars(session)),
        parameters
    )

    try:
        assert_success(response)
    except AssertionError:
        # > 4. If parameters.state is an inappropriate permission state for any
        # >    implementation-defined reason, return a WebDriver error with
        # >    WebDriver error code invalid argument.
        assert_error(response, "invalid argument")
        return

    assert response.body.get("value") == None

    session.window_handle = original_window

    response = query(session, "geolocation")

    assert_success(response)
    result = response.body.get("value")

    assert isinstance(result, dict)
    assert result.get("status") == "success"
    assert result.get("value") == state

# The following test is not implemented because UAs may vary in the way they
# modify permissions across realms, so the behavior of the `oneRealm` parameter
# cannot be asserted uniformly.
# def test_set_to_state_one_realm():
#     pass
