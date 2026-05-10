import pytest

from tests.support.classic.asserts import assert_success

from . import set_user_verified


@pytest.mark.parametrize("value", [True, False])
def test_set_user_verified(session, authenticator, value):
    response = set_user_verified(session, authenticator, value)
    assert_success(response)
