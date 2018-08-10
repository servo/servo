import uuid

import pytest

from tests.support.asserts import assert_success


def test_sessionid(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert isinstance(value["sessionId"], basestring)
    uuid.UUID(hex=value["sessionId"])


@pytest.mark.parametrize("capability, type", [
    ("browserName", basestring),
    ("browserVersion", basestring),
    ("platformName", basestring),
    ("acceptInsecureCerts", bool),
    ("pageLoadStrategy", basestring),
    ("proxy", dict),
    ("setWindowRect", bool),
    ("timeouts", dict),
    ("unhandledPromptBehavior", basestring),
])
def test_capability_type(session, capability, type):
    assert isinstance(session.capabilities, dict)
    assert capability in session.capabilities
    assert isinstance(session.capabilities[capability], type)


@pytest.mark.parametrize("capability, default_value", [
    ("acceptInsecureCerts", False),
    ("pageLoadStrategy", "normal"),
    ("proxy", {}),
    ("setWindowRect", True),
    ("timeouts", {"implicit": 0, "pageLoad": 300000, "script": 30000}),
    ("unhandledPromptBehavior", "dismiss and notify"),
])
def test_capability_default_value(session, capability, default_value):
    assert isinstance(session.capabilities, dict)
    assert capability in session.capabilities
    assert session.capabilities[capability] == default_value
