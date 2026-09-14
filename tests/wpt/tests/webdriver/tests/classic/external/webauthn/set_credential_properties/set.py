import pytest

from tests.support.classic.asserts import assert_success

from .. import create_credential
from . import set_credential_properties


@pytest.mark.parametrize("backup_eligibility", [True, False])
def test_set_backup_eligibility(session, authenticator, backup_eligibility):
    credential = create_credential(credential_id="cHJvcHMtMQ")
    session.web_authn.add_credential(authenticator, credential)

    response = set_credential_properties(
        session, authenticator, "cHJvcHMtMQ", {"backupEligibility": backup_eligibility}
    )
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["backupEligibility"] is backup_eligibility


@pytest.mark.parametrize("backup_state", [True, False])
def test_set_backup_state(session, authenticator, backup_state):
    credential = create_credential(credential_id="cHJvcHMtMg")
    session.web_authn.add_credential(authenticator, credential)

    response = set_credential_properties(
        session, authenticator, "cHJvcHMtMg", {"backupState": backup_state}
    )
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["backupState"] is backup_state


@pytest.mark.parametrize("sign_count", [0, 42, None, 2**32 - 1])
def test_set_sign_count(session, authenticator, sign_count):
    credential = create_credential(credential_id="cHJvcHMtMw", sign_count=1)
    session.web_authn.add_credential(authenticator, credential)

    response = set_credential_properties(
        session, authenticator, "cHJvcHMtMw", {"signCount": sign_count}
    )
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["signCount"] == sign_count


def test_set_multiple_properties(session, authenticator):
    credential = create_credential(credential_id="cHJvcHMtNA", sign_count=1)
    session.web_authn.add_credential(authenticator, credential)

    properties = {
        "backupEligibility": True,
        "backupState": True,
        "signCount": 100,
    }
    response = set_credential_properties(
        session, authenticator, "cHJvcHMtNA", properties
    )
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["backupEligibility"] is True
    assert credentials[0]["backupState"] is True
    assert credentials[0]["signCount"] == 100
