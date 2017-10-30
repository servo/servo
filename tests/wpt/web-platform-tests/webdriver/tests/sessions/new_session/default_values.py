# META: timeout=long

import uuid

import pytest

from webdriver import error


def test_basic(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    assert set(resp.keys()) == {"sessionId", "capabilities"}


def test_repeat_new_session(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    with pytest.raises(error.SessionNotCreatedException):
        new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})


def test_no_capabilites(new_session):
    with pytest.raises(error.InvalidArgumentException):
        new_session({})


def test_missing_first_match(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})


def test_missing_always_match(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"firstMatch": [add_browser_capabilites({})]}})


def test_desired(new_session, add_browser_capabilites):
    with pytest.raises(error.InvalidArgumentException):
        resp, _ = new_session({"desiredCapbilities": add_browser_capabilites({})})


def test_ignore_non_spec_fields_in_capabilities(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({}), "desiredCapbilities": {"pageLoadStrategy": "eager"}}})
    assert resp["capabilities"]["pageLoadStrategy"] == "normal"


def test_valid_but_unmatchable_key(new_session, add_browser_capabilites):
    resp, _ = new_session({"capabilities": {
      "firstMatch": [add_browser_capabilites({"pageLoadStrategy": "eager", "foo:unmatchable": True}),
                     {"pageLoadStrategy": "none"}]}})
    assert resp["capabilities"]["pageLoadStrategy"] == "none"
