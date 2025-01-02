import pytest

from tests.support.asserts import assert_success


@pytest.mark.parametrize("value", [None, False])
def test_no_bidi_upgrade(new_session, add_browser_capabilities, value):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({"webSocketUrl": value})}})
    value = assert_success(response)
    assert value["capabilities"].get("webSocketUrl") == None


def test_bidi_upgrade(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({"webSocketUrl": True})}})
    value = assert_success(response)
    assert value["capabilities"]["webSocketUrl"].startswith("ws://")
    assert value["sessionId"] in value["capabilities"]["webSocketUrl"]
