#META: timeout=long

import pytest

from conftest import product, flatten

from .support.create import valid_data


@pytest.mark.parametrize("key,value", flatten(product(*item) for item in valid_data))
def test_valid(new_session, key, value):
    resp = new_session({"capabilities": {"alwaysMatch": {key: value}}})

