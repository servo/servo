# META: timeout=long

from tests.support.asserts import assert_error, assert_success


def test_basic(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert set(value.keys()) == {"sessionId", "capabilities"}


def test_repeat_new_session(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    assert_success(response)

    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    assert_error(response, "session not created")


def test_no_capabilites(new_session):
    response, _ = new_session({})
    assert_error(response, "invalid argument")


def test_missing_first_match(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    assert_success(response)


def test_missing_always_match(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"firstMatch": [add_browser_capabilities({})]}})
    assert_success(response)


def test_desired(new_session, add_browser_capabilities):
    response, _ = new_session({"desiredCapbilities": add_browser_capabilities({})})
    assert_error(response, "invalid argument")


def test_ignore_non_spec_fields_in_capabilities(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({}),
        "desiredCapbilities": {"pageLoadStrategy": "eager"},
    }})
    value = assert_success(response)
    assert value["capabilities"]["pageLoadStrategy"] == "normal"


def test_valid_but_unmatchable_key(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"firstMatch": [
        add_browser_capabilities({"pageLoadStrategy": "eager", "foo:unmatchable": True}),
        {"pageLoadStrategy": "none"},
    ]}})
    value = assert_success(response)
    assert value["capabilities"]["pageLoadStrategy"] == "none"
