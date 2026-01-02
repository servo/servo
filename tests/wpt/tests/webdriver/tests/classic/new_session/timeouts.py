import pytest

from tests.support.asserts import assert_success, assert_error

MAX_SAFE_INTEGER = 2**53 - 1

def test_default_values(session):
    timeouts = session.capabilities["timeouts"]

    assert timeouts["implicit"] == 0
    assert timeouts["pageLoad"] == 300000
    assert timeouts["script"] == 30000


@pytest.mark.parametrize("timeouts", [
    {"implicit": 444, "pageLoad": 300000,"script": 30000},
    {"implicit": 0, "pageLoad": None,"script": 30000},
    {"implicit": None, "pageLoad": 300000,"script": 444},
    {"implicit": 0, "pageLoad": 300000,"script": None},
])
def test_timeouts(new_session, add_browser_capabilities, timeouts):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({"timeouts": timeouts})}})
    value = assert_success(response)
    assert value["capabilities"]["timeouts"] == timeouts

@pytest.mark.parametrize("timeouts", [
    {"implicit": None, "pageLoad": MAX_SAFE_INTEGER + 2,"script": 30000},
    {"implicit": MAX_SAFE_INTEGER + 2, "pageLoad": None,"script": 30000},
    {"implicit": None, "pageLoad": None,"script": MAX_SAFE_INTEGER + 2}
])
def test_invalid_timeouts(new_session, add_browser_capabilities, timeouts):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({"timeouts": timeouts})}})
    assert_error(response, "invalid argument")
