# META: timeout=long

import uuid

from tests.support.asserts import assert_success


def test_sessionid(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert isinstance(value["sessionId"], basestring)
    uuid.UUID(hex=value["sessionId"])


def test_capabilites(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert isinstance(value["capabilities"], dict)

    all_capabilities = set(value["capabilities"].keys())
    expected_capabilities = {
        "browserName",
        "browserVersion",
        "platformName",
        "acceptInsecureCerts",
        "setWindowRect",
        "timeouts",
        "proxy",
        "pageLoadStrategy",
    }

    assert expected_capabilities.issubset(all_capabilities), (
        "{0} cannot be found in {1}".format(
            list(expected_capabilities - all_capabilities), all_capabilities))


def test_data(new_session, add_browser_capabilities, platform_name):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)

    assert isinstance(value["capabilities"]["browserName"], basestring)
    assert isinstance(value["capabilities"]["browserVersion"], basestring)
    if platform_name:
        assert value["capabilities"]["platformName"] == platform_name
    else:
        assert "platformName" in value["capabilities"]
    assert value["capabilities"]["acceptInsecureCerts"] is False
    assert isinstance(value["capabilities"]["setWindowRect"], bool)
    assert value["capabilities"]["timeouts"]["implicit"] == 0
    assert value["capabilities"]["timeouts"]["pageLoad"] == 300000
    assert value["capabilities"]["timeouts"]["script"] == 30000
    assert value["capabilities"]["proxy"] == {}
    assert value["capabilities"]["pageLoadStrategy"] == "normal"


def test_timeouts(new_session, add_browser_capabilities, platform_name):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({"timeouts": {"implicit": 1000}}),
    }})
    value = assert_success(response)

    assert value["capabilities"]["timeouts"] == {
        "implicit": 1000,
        "pageLoad": 300000,
        "script": 30000
    }


def test_pageLoadStrategy(new_session, add_browser_capabilities, platform_name):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({"pageLoadStrategy": "eager"})}})
    value = assert_success(response)

    assert value["capabilities"]["pageLoadStrategy"] == "eager"
