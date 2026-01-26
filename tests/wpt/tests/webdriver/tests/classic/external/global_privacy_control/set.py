import pytest

from tests.support.asserts import assert_error, assert_success
from . import get_gpc, set_gpc


@pytest.mark.parametrize("value", [True, False])
def test_set_gpc_success(session, value):
    response = set_gpc(session, value)
    newValue = assert_success(response)

    assert "gpc" in newValue
    assert type(newValue["gpc"]) is bool
    assert newValue["gpc"] == value

    getResponse = get_gpc(session)
    value = assert_success(getResponse)

    assert "gpc" in value
    assert type(value["gpc"]) is bool
    assert newValue["gpc"] == value["gpc"]


@pytest.mark.parametrize("value", [None, 1, "hello", [], {}])
def test_set_gpc_failure(session, value):
    response = set_gpc(session, value)
    assert_error(response, "invalid argument")
