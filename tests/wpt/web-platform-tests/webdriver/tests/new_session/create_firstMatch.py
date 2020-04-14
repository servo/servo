# META: timeout=long

import pytest

from .conftest import product, flatten


from tests.support.asserts import assert_success
from tests.new_session.support.create import valid_data


@pytest.mark.parametrize("key,value", flatten(product(*item) for item in valid_data))
def test_valid(new_session, add_browser_capabilities, key, value):
    response, _ = new_session({"capabilities": {
        "firstMatch": [add_browser_capabilities({key: value})]}})
    assert_success(response)
