import pytest

from tests.support import platform_name
from tests.support.asserts import assert_success


@pytest.mark.skipif(platform_name is None, reason="Unsupported platform {}".format(platform_name))
def test_corresponds_to_local_system(new_session, add_browser_capabilities):
    response, _ = new_session({"capabilities": {"alwaysMatch": add_browser_capabilities({})}})
    value = assert_success(response)
    assert value["capabilities"]["platformName"] == platform_name
