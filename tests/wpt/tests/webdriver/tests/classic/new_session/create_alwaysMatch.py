# META: timeout=long

import pytest

from .conftest import product, flatten

from tests.support.asserts import assert_success
from tests.classic.new_session.support.create import valid_data


@pytest.mark.parametrize("key,value", flatten(product(*item) for item in valid_data))
def test_valid(new_session, add_browser_capabilities, key, value):
    response, _ = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilities({key: value})}})
    assert_success(response)
    response_capabilities = response.body["value"]["capabilities"]
    if ":" not in key and value is not None:
        if key == "timeouts":
            for timeout_key, timeout_value in value.items():
                assert response_capabilities[key][timeout_key] == timeout_value
        else:
            assert response_capabilities[key] == value
