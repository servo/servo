import pytest
from tests.support.classic.asserts import assert_error

from .. import create_credential
from . import set_credential_properties


def test_authenticator_id_invalid_value(session):
    response = set_credential_properties(
        session, authenticator_id="invalid", credential_id="Y3JlZC0x", properties={}
    )
    assert_error(response, "invalid argument")


def test_credential_id_invalid_value(session, authenticator):
    response = set_credential_properties(
        session,
        authenticator_id=authenticator,
        credential_id="invalid",
        properties={},
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("backup_eligibility", ["foo", 123, [], {}, None])
def test_backup_eligibility_invalid_type(session, authenticator, backup_eligibility):
    credential = create_credential(credential_id="Y3JlZC0x")
    session.web_authn.add_credential(authenticator, credential)

    response = set_credential_properties(
        session,
        authenticator,
        "Y3JlZC0x",
        {"backupEligibility": backup_eligibility},
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("backup_state", ["foo", 123, [], {}, None])
def test_backup_state_invalid_type(session, authenticator, backup_state):
    credential = create_credential(credential_id="Y3JlZC0x")
    session.web_authn.add_credential(authenticator, credential)

    response = set_credential_properties(
        session,
        authenticator,
        "Y3JlZC0x",
        {"backupState": backup_state},
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("sign_count", ["foo", True, [], {}, -1, 1.5, 2**32])
def test_sign_count_invalid_type(session, authenticator, sign_count):
    credential = create_credential(credential_id="Y3JlZC0x")
    session.web_authn.add_credential(authenticator, credential)

    response = set_credential_properties(
        session,
        authenticator,
        "Y3JlZC0x",
        {"signCount": sign_count},
    )
    assert_error(response, "invalid argument")
