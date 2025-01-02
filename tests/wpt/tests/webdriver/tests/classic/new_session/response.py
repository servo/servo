import uuid
import pytest

from tests.support.asserts import assert_success


def test_sessionid(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert isinstance(value["sessionId"], str)
    uuid.UUID(hex=value["sessionId"])


@pytest.mark.parametrize("capability, type", [
    ("browserName", str),
    ("browserVersion", str),
    ("platformName", str),
    ("acceptInsecureCerts", bool),
    ("pageLoadStrategy", str),
    ("proxy", dict),
    ("setWindowRect", bool),
    ("timeouts", dict),
    ("strictFileInteractability", bool),
    ("unhandledPromptBehavior", str),
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
    ("strictFileInteractability", False),
    ("unhandledPromptBehavior", "dismiss and notify"),
])
def test_capability_default_value(session, capability, default_value):
    assert isinstance(session.capabilities, dict)
    assert capability in session.capabilities
    assert session.capabilities[capability] == default_value


def test_capability_user_agent(session):
    assert isinstance(session.capabilities, dict)
    assert "userAgent" in session.capabilities
    assert type(session.capabilities["userAgent"]) == str
    assert len(session.capabilities["userAgent"]) > 0
