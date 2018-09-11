import pytest

from conftest import product, flatten

from tests.new_session.support.create import invalid_data, invalid_extensions
from tests.support.asserts import assert_error


@pytest.mark.parametrize("value", [None, 1, "{}", []])
def test_invalid_capabilites(new_session, value):
    response, _ = new_session({"capabilities": value})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, 1, "{}", []])
def test_invalid_always_match(new_session, add_browser_capabilities, value):
    capabilities = {"alwaysMatch": value, "firstMatch": [add_browser_capabilities({})]}

    response, _ = new_session({"capabilities": capabilities})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, 1, "[]", {}])
def test_invalid_first_match(new_session, add_browser_capabilities, value):
    capabilities = {"alwaysMatch": add_browser_capabilities({}), "firstMatch": value}

    response, _ = new_session({"capabilities": capabilities})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("body", [lambda key, value: {"alwaysMatch": {key: value}},
                                  lambda key, value: {"firstMatch": [{key: value}]}])
@pytest.mark.parametrize("key,value", flatten(product(*item) for item in invalid_data))
def test_invalid_values(new_session, add_browser_capabilities, body, key, value):
    capabilities = body(key, value)
    if "alwaysMatch" in capabilities:
        capabilities["alwaysMatch"] = add_browser_capabilities(capabilities["alwaysMatch"])
    else:
        capabilities["firstMatch"][0] = add_browser_capabilities(capabilities["firstMatch"][0])

    response, _ = new_session({"capabilities": capabilities})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("body", [lambda key, value: {"alwaysMatch": {key: value}},
                                  lambda key, value: {"firstMatch": [{key: value}]}])
@pytest.mark.parametrize("key", invalid_extensions)
def test_invalid_extensions(new_session, add_browser_capabilities, body, key):
    capabilities = body(key, {})
    if "alwaysMatch" in capabilities:
        capabilities["alwaysMatch"] = add_browser_capabilities(capabilities["alwaysMatch"])
    else:
        capabilities["firstMatch"][0] = add_browser_capabilities(capabilities["firstMatch"][0])

    response, _ = new_session({"capabilities": capabilities})
    assert_error(response, "invalid argument")
