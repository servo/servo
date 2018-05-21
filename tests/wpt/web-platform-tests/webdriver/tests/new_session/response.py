# META: timeout=long

import uuid

def test_resp_sessionid(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    assert isinstance(resp["sessionId"], unicode)
    uuid.UUID(hex=resp["sessionId"])


def test_resp_capabilites(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    assert isinstance(resp["sessionId"], unicode)
    assert isinstance(resp["capabilities"], dict)
    assert {"browserName",
            "browserVersion",
            "platformName",
            "acceptInsecureCerts",
            "setWindowRect",
            "timeouts",
            "proxy",
            "pageLoadStrategy"}.issubset(
                set(resp["capabilities"].keys()))


def test_resp_data(new_session, add_browser_capabilites, platform_name):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})

    assert isinstance(resp["capabilities"]["browserName"], unicode)
    assert isinstance(resp["capabilities"]["browserVersion"], unicode)
    if platform_name:
        assert resp["capabilities"]["platformName"] == platform_name
    else:
        assert "platformName" in resp["capabilities"]
    assert resp["capabilities"]["acceptInsecureCerts"] is False
    assert isinstance(resp["capabilities"]["setWindowRect"], bool)
    assert resp["capabilities"]["timeouts"]["implicit"] == 0
    assert resp["capabilities"]["timeouts"]["pageLoad"] == 300000
    assert resp["capabilities"]["timeouts"]["script"] == 30000
    assert resp["capabilities"]["proxy"] == {}
    assert resp["capabilities"]["pageLoadStrategy"] == "normal"


def test_timeouts(new_session, add_browser_capabilites, platform_name):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"timeouts": {"implicit": 1000}})}})
    assert resp["capabilities"]["timeouts"] == {
        "implicit": 1000,
        "pageLoad": 300000,
        "script": 30000
    }

def test_pageLoadStrategy(new_session, add_browser_capabilites, platform_name):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"pageLoadStrategy": "eager"})}})
    assert resp["capabilities"]["pageLoadStrategy"] == "eager"
