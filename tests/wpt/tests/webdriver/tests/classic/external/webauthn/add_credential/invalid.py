import pytest
from tests.support.classic.asserts import assert_error

from .. import create_credential
from . import add_credential


def test_authenticator_id_invalid_value(session):
    response = add_credential(session, "invalid", create_credential())
    assert_error(response, "invalid argument")


def test_empty_parameters(session, authenticator):
    response = add_credential(session, authenticator, {})
    assert_error(response, "invalid argument")


def test_credential_id_missing(session, authenticator):
    credential = create_credential()
    del credential["credentialId"]
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("credential_id", [None, 123, True, [], {}])
def test_credential_id_invalid_type(session, authenticator, credential_id):
    credential = create_credential()
    credential["credentialId"] = credential_id
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


def test_is_resident_credential_missing(session, authenticator):
    credential = create_credential()
    del credential["isResidentCredential"]
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("is_resident_credential", [None, "true", 123, [], {}])
def test_is_resident_credential_invalid_type(
    session, authenticator, is_resident_credential
):
    credential = create_credential()
    credential["isResidentCredential"] = is_resident_credential
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


def test_rp_id_missing(session, authenticator):
    credential = create_credential()
    del credential["rpId"]
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("rp_id", [None, 123, True, [], {}])
def test_rp_id_invalid_type(session, authenticator, rp_id):
    credential = create_credential()
    credential["rpId"] = rp_id
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


def test_private_key_missing(session, authenticator):
    credential = create_credential()
    del credential["privateKey"]
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("private_key", [None, 123, True, [], {}])
def test_private_key_invalid_type(session, authenticator, private_key):
    credential = create_credential()
    credential["privateKey"] = private_key
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


def test_sign_count_missing(session, authenticator):
    credential = create_credential()
    del credential["signCount"]
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("sign_count", [None, "zero", True, [], {}, -1])
def test_sign_count_invalid_type(session, authenticator, sign_count):
    credential = create_credential()
    credential["signCount"] = sign_count
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("user_handle", [123, True, [], {}])
def test_user_handle_invalid_type(session, authenticator, user_handle):
    credential = create_credential()
    credential["userHandle"] = user_handle
    response = add_credential(session, authenticator, credential)
    assert_error(response, "invalid argument")
