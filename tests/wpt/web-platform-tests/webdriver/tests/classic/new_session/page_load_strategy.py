from tests.support.asserts import assert_success

def test_pageLoadStrategy(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({"pageLoadStrategy": "eager"})}})
    value = assert_success(response)
    assert value["capabilities"]["pageLoadStrategy"] == "eager"
