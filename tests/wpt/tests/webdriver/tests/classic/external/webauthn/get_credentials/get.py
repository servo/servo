from tests.support.classic.asserts import assert_success

from .. import create_credential
from . import get_credentials


def test_get_credentials_none(session, authenticator):
    response = get_credentials(session, authenticator)
    credentials = assert_success(response)

    assert isinstance(credentials, list)
    assert len(credentials) == 0


def test_get_credentials_single(session, authenticator):
    credential = create_credential()
    session.web_authn.add_credential(authenticator, credential)

    response = get_credentials(session, authenticator)
    credentials = assert_success(response)

    assert isinstance(credentials, list)
    assert len(credentials) == 1
    assert credentials[0]["credentialId"] == "Y3JlZC0x"
    assert credentials[0]["isResidentCredential"] is False
    assert credentials[0]["rpId"] == "example.com"
    assert credentials[0]["signCount"] == 0


def test_get_credentials_multiple(session, authenticator):
    credential_ids = ["Y3JlZC0w", "Y3JlZC0x", "Y3JlZC0y"]
    for credential_id in credential_ids:
        session.web_authn.add_credential(
            authenticator, create_credential(credential_id=credential_id)
        )

    response = get_credentials(session, authenticator)
    credentials = assert_success(response)

    assert isinstance(credentials, list)
    assert len(credentials) == 3

    returned_ids = {credential["credentialId"] for credential in credentials}
    assert returned_ids == set(credential_ids)


def test_get_credentials_resident(session, authenticator):
    credential = create_credential(
        credential_id="cmVzaWRlbnQ",
        is_resident_credential=True,
        sign_count=10,
    )
    session.web_authn.add_credential(authenticator, credential)

    response = get_credentials(session, authenticator)
    credentials = assert_success(response)

    assert isinstance(credentials, list)
    assert len(credentials) == 1
    assert credentials[0]["credentialId"] == "cmVzaWRlbnQ"
    assert credentials[0]["isResidentCredential"] is True
    assert credentials[0]["signCount"] == 10
