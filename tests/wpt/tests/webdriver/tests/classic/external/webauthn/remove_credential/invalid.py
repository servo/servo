from tests.support.classic.asserts import assert_error

from . import remove_credential


def test_authenticator_id_invalid_value(session):
    response = remove_credential(
        session, authenticator_id="invalid", credential_id="Y3JlZC0x"
    )
    assert_error(response, "invalid argument")


def test_credential_id_invalid_value(session, authenticator):
    response = remove_credential(
        session, authenticator_id=authenticator, credential_id="invalid"
    )
    assert_error(response, "invalid argument")
