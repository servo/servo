#META: timeout=long

import pytest

from conftest import product, flatten


# Note that we can only test things here all implementations must support
valid_data = [
    ("acceptInsecureCerts", [False, None]),
    ("browserName", [None]),
    ("browserVersion", [None]),
    ("platformName", [None]),
    ("pageLoadStrategy", ["none", "eager", "normal", None]),
    ("proxy", [None]),
    ("unhandledPromptBehavior", ["dismiss", "accept", None]),
    ("test:extension", [True, "abc", 123, [], {"key": "value"}, None]),
]


@pytest.mark.parametrize("body", [lambda key, value: {"alwaysMatch": {key: value}},
                                  lambda key, value: {"firstMatch": [{key: value}]}])
@pytest.mark.parametrize("key,value", flatten(product(*item) for item in valid_data))
def test_valid(new_session, body, key, value):
    resp = new_session({"capabilities": body(key, value)})

# Continued in create-1.py to avoid timeouts
