from tests.support.classic.asserts import assert_success

from .. import create_credential
from . import remove_credential


def test_remove_credential(session, authenticator):
    credential = create_credential(credential_id="Y3JlZC0w")
    session.web_authn.add_credential(authenticator, credential)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1

    response = remove_credential(session, authenticator, "Y3JlZC0w")
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 0


def test_remove_credential_from_multiple(session, authenticator):
    for credential_id in ["Y3JlZC0w", "Y3JlZC0x", "Y3JlZC0y"]:
        session.web_authn.add_credential(
            authenticator, create_credential(credential_id=credential_id)
        )

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 3

    response = remove_credential(session, authenticator, "Y3JlZC0x")
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 2

    returned_ids = {c["credentialId"] for c in credentials}
    assert returned_ids == {"Y3JlZC0w", "Y3JlZC0y"}
