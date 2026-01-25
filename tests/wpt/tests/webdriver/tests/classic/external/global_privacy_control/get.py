from tests.support.asserts import assert_success

from . import get_gpc


def test_get_gpc(session):
    response = get_gpc(session)
    value = assert_success(response)

    assert "gpc" in value
    assert type(value["gpc"]) is bool
