from tests.support.asserts import assert_error

# Passing no capabilities to the webdriver executable can cause various
# side-effects. As such this particular test should be run separately.

def test_no_capabilites(new_session):
    response, _ = new_session({})
    assert_error(response, "invalid argument")
