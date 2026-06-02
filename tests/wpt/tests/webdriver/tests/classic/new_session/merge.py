# META: timeout=long

import pytest

from tests.support.asserts import assert_error, assert_success


@pytest.mark.parametrize("body", [
    lambda key, value: {"alwaysMatch": {key: value}},
    lambda key, value: {"firstMatch": [{key: value}]}
], ids=["alwaysMatch", "firstMatch"])
def test_platform_name(new_session, add_browser_capabilities, body, target_platform):
    capabilities = body("platformName", target_platform)
    if "alwaysMatch" in capabilities:
        capabilities["alwaysMatch"] = add_browser_capabilities(capabilities["alwaysMatch"])
    else:
        capabilities["firstMatch"][0] = add_browser_capabilities(capabilities["firstMatch"][0])

    response, _ = new_session({"capabilities": capabilities})
    value = assert_success(response)

    assert value["capabilities"]["platformName"] == target_platform


invalid_merge = [
    ("acceptInsecureCerts", (True, True)),
    ("unhandledPromptBehavior", ("accept", "accept")),
    ("unhandledPromptBehavior", ("accept", "dismiss")),
    ("timeouts", ({"script": 10}, {"script": 10})),
    ("timeouts", ({"script": 10}, {"pageLoad": 10})),
]


@pytest.mark.parametrize("key,value", invalid_merge)
def test_merge_invalid(new_session, add_browser_capabilities, key, value):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({key: value[0]}),
        "firstMatch": [{}, {key: value[1]}],
    }})
    assert_error(response, "invalid argument")


def test_merge_platform_name(new_session, add_browser_capabilities, target_platform):
    always_match = add_browser_capabilities({})

    if "platformName" in always_match:
        platform_name = always_match.pop("platformName")
    else:
        platform_name = target_platform

    # Remove pageLoadStrategy so we can use it to validate the merging of the firstMatch
    # capabilities, and guarantee platformName isn't simply ignored.
    if "pageLoadStrategy" in always_match:
        del always_match["pageLoadStrategy"]

    response, _ = new_session({"capabilities": {
        "alwaysMatch": always_match,
        "firstMatch": [{
            "platformName": platform_name.upper(),
            "pageLoadStrategy": "none",
        }, {
            "platformName": platform_name,
            "pageLoadStrategy": "eager",
        }]}})

    value = assert_success(response)

    assert value["capabilities"]["platformName"] == platform_name
    assert value["capabilities"]["pageLoadStrategy"] == "eager"


def test_merge_browserName(new_session, add_browser_capabilities):
    always_match = add_browser_capabilities({})

    if "browserName" in always_match:
        browser_name = always_match.pop("browserName")
    else:
        response, _ = new_session(
            {"capabilities": {"alwaysMatch": always_match}})
        value = assert_success(response)
        browser_name = value["capabilities"]["browserName"]

    # Remove pageLoadStrategy so we can use it to validate the merging of the firstMatch
    # capabilities, and guarantee browserName isn't simply ignored.
    if "pageLoadStrategy" in always_match:
        del always_match["pageLoadStrategy"]

    response, _ = new_session({"capabilities": {
        "alwaysMatch": always_match,
        "firstMatch": [{
            "browserName": browser_name + "invalid",
            "pageLoadStrategy": "none",
        }, {
            "browserName": browser_name,
            "pageLoadStrategy": "eager",
        }]}}, delete_existing_session=True)

    value = assert_success(response)

    assert value["capabilities"]["browserName"] == browser_name
    assert value["capabilities"]["pageLoadStrategy"] == "eager"
