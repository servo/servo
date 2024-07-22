from tests.support.asserts import assert_success


def test_corresponds_to_local_system(new_session, add_browser_capabilities, target_platform):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert value["capabilities"]["platformName"] == target_platform
