#META: timeout=long

import pytest
from webdriver import error

from conftest import platform_name


@pytest.mark.skipif(platform_name() is None, reason="Unsupported platform")
@pytest.mark.parametrize("body", [lambda key, value: {"alwaysMatch": {key: value}},
                                  lambda key, value: {"firstMatch": [{key: value}]}])
def test_platform_name(new_session, add_browser_capabilites, platform_name, body):
    capabilities = body("platformName", platform_name)
    if "alwaysMatch" in capabilities:
        capabilities["alwaysMatch"] = add_browser_capabilites(capabilities["alwaysMatch"])
    else:
        capabilities["firstMatch"][0] = add_browser_capabilites(capabilities["firstMatch"][0])
    resp, _ = new_session({"capabilities": capabilities})
    assert resp["capabilities"]["platformName"] == platform_name


invalid_merge = [
    ("acceptInsecureCerts", (True, True)),
    ("unhandledPromptBehavior", ("accept", "accept")),
    ("unhandledPromptBehavior", ("accept", "dismiss")),
    ("timeouts", ({"script": 10}, {"script": 10})),
    ("timeouts", ({"script": 10}, {"pageLoad": 10})),
]


@pytest.mark.parametrize("key,value", invalid_merge)
def test_merge_invalid(new_session, add_browser_capabilites, key, value):
    with pytest.raises(error.InvalidArgumentException):
         new_session({"capabilities":
                      {"alwaysMatch": add_browser_capabilites({key: value[0]}),
                       "firstMatch": [{}, {key: value[1]}]}})


@pytest.mark.skipif(platform_name() is None, reason="Unsupported platform")
def test_merge_platformName(new_session, add_browser_capabilites, platform_name):
    resp, _ = new_session({"capabilities":
                        {"alwaysMatch": add_browser_capabilites({"timeouts": {"script": 10}}),
                        "firstMatch": [
                            {
                                "platformName": platform_name.upper(),
                                "pageLoadStrategy": "none"
                            },
                            {
                                "platformName": platform_name,
                                "pageLoadStrategy": "eager"
                            }
                        ]}})

    assert resp["capabilities"]["platformName"] == platform_name
    assert resp["capabilities"]["pageLoadStrategy"] == "eager"


def test_merge_browserName(new_session, add_browser_capabilites):
    resp, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    browser_settings = {
        "browserName": resp["capabilities"]["browserName"],
        "browserVersion": resp["capabilities"]["browserVersion"],
        "platformName": resp["capabilities"]["platformName"]
    }
    session.end()

    resp, _ = new_session({"capabilities":
                        {"alwaysMatch": add_browser_capabilites({"timeouts": {"script": 10}}),
                        "firstMatch": [
                            {
                                "browserName": browser_settings["browserName"] + "invalid",
                                "pageLoadStrategy": "none"
                            },
                            {
                                "browserName": browser_settings["browserName"],
                                "pageLoadStrategy": "eager"
                            }
                        ]}})

    assert resp["capabilities"]["browserName"] == browser_settings['browserName']
    assert resp["capabilities"]["pageLoadStrategy"] == "eager"
