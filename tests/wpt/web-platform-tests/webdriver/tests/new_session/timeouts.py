import pytest

from tests.support.asserts import assert_success


def test_default_values(session):
    timeouts = session.capabilities["timeouts"]

    assert timeouts["implicit"] == 0
    assert timeouts["pageLoad"] == 300000
    assert timeouts["script"] == 30000


@pytest.mark.parametrize("timeouts", [
    {"implicit": 444, "pageLoad": 300000,"script": 30000},
    {"implicit": 0, "pageLoad": 444,"script": 30000},
    {"implicit": 0, "pageLoad": 300000,"script": 444},
])
def test_timeouts(new_session, add_browser_capabilities, timeouts):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({"timeouts": timeouts})}})
    value = assert_success(response)
    assert value["capabilities"]["timeouts"] == timeouts
