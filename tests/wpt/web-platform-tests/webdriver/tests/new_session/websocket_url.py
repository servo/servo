from tests.support.asserts import assert_success

def test_websocket_url(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({"webSocketUrl": True})}})
    value = assert_success(response)
    assert value["capabilities"]["webSocketUrl"].startswith("ws://")
