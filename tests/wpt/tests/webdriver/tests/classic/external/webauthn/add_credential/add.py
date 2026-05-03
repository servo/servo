from tests.support.classic.asserts import assert_success

from .. import create_credential
from . import add_credential


def test_add_credential(session, authenticator):
    response = add_credential(session, authenticator, create_credential())
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["credentialId"] == "Y3JlZC0x"
    assert credentials[0]["isResidentCredential"] is False
    assert credentials[0]["rpId"] == "example.com"
    assert credentials[0]["signCount"] == 0


def test_add_resident_credential(session, authenticator):
    credential = create_credential(
        credential_id="cmVzaWRlbnQ",
        is_resident_credential=True,
        user_handle="dXNlcjE",
    )

    response = add_credential(session, authenticator, credential)
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["credentialId"] == "cmVzaWRlbnQ"
    assert credentials[0]["isResidentCredential"] is True
    assert credentials[0]["userHandle"] == "dXNlcjE"


def test_add_credential_with_large_blob(session, authenticator):
    credential = create_credential(
        credential_id="bGFyZ2VibG9i",
        large_blob="c29tZSBibG9i",
    )

    response = add_credential(session, authenticator, credential)
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["credentialId"] == "bGFyZ2VibG9i"
    assert credentials[0]["largeBlob"] == "c29tZSBibG9i"


def test_add_credential_with_sign_count(session, authenticator):
    credential = create_credential(credential_id="c2lnbmNvdW50", sign_count=42)

    response = add_credential(session, authenticator, credential)
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1
    assert credentials[0]["credentialId"] == "c2lnbmNvdW50"
    assert credentials[0]["signCount"] == 42


def test_add_multiple_credentials(session, authenticator):
    credential_ids = ["Y3JlZC0w", "Y3JlZC0x", "Y3JlZC0y"]
    for credential_id in credential_ids:
        response = add_credential(
            session,
            authenticator,
            create_credential(credential_id=credential_id)
        )
        assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 3

    returned_ids = {c["credentialId"] for c in credentials}
    assert returned_ids == set(credential_ids)
