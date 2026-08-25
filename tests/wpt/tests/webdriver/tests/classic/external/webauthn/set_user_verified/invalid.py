import pytest

from tests.support.classic.asserts import assert_error

from . import set_user_verified


def test_authenticator_id_invalid_value(session):
    response = set_user_verified(session, authenticator_id="invalid", is_user_verified=True)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("is_user_verified", [None, "true", 123, [], {}])
def test_is_user_verified_invalid_type(session, authenticator, is_user_verified):
    response = set_user_verified(session, authenticator, is_user_verified)
    assert_error(response, "invalid argument")
