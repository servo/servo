from tests.support.classic.asserts import assert_error

from . import remove_all_credentials


def test_authenticator_id_invalid_value(session):
    response = remove_all_credentials(session, authenticator_id="invalid")
    assert_error(response, "invalid argument")
