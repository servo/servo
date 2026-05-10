from tests.support.classic.asserts import assert_error

from . import get_credentials


def test_authenticator_id_invalid_value(session):
    response = get_credentials(session, authenticator_id="invalid")
    assert_error(response, "invalid argument")
