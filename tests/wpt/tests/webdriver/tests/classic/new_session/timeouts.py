import pytest

from tests.support.asserts import assert_success, assert_error

MAX_SAFE_INTEGER = 2**53 - 1

def test_default_values(session):
    timeouts = session.capabilities["timeouts"]

    assert timeouts["implicit"] == 0
    assert timeouts["pageLoad"] == 300000
    assert timeouts["script"] == 30000


@pytest.mark.parametrize("value", [None, 0, 3000])
@pytest.mark.parametrize("key", ["implicit", "pageLoad", "script"])
def test_timeouts(new_session, add_browser_capabilities, key, value):
    timeouts = {key: value}
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({"timeouts": timeouts})}})
    value = assert_success(response)
    assert value["capabilities"]["timeouts"] == timeouts

@pytest.mark.parametrize("value", [MAX_SAFE_INTEGER + 1, -1])
@pytest.mark.parametrize("key", ["implicit", "pageLoad", "script"])
def test_invalid_timeouts_value(new_session, add_browser_capabilities, key, value):
    timeouts = {key: value}

    response, _ = new_session({
        "capabilities": {
            "alwaysMatch": add_browser_capabilities({"timeouts": timeouts})
        }
    })

    assert_error(response, "invalid argument")

@pytest.mark.parametrize("value", ["foo", False, [], {}])
@pytest.mark.parametrize("key", ["implicit", "pageLoad", "script"])
def test_invalid_timeouts_type(new_session, add_browser_capabilities, key, value):
    timeouts = {key: value}

    response, _ = new_session({
        "capabilities": {
            "alwaysMatch": add_browser_capabilities({"timeouts": timeouts})
        }
    })

    assert_error(response, "invalid argument")
