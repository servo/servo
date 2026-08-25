from tests.support.classic.asserts import assert_success

from .. import create_credential
from . import remove_all_credentials


def test_remove_all_credentials_empty(session, authenticator):
    response = remove_all_credentials(session, authenticator)
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 0


def test_remove_all_credentials_single(session, authenticator):
    session.web_authn.add_credential(authenticator, create_credential())

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 1

    response = remove_all_credentials(session, authenticator)
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 0


def test_remove_all_credentials_multiple(session, authenticator):
    for credential_id in ["Y3JlZC0w", "Y3JlZC0x", "Y3JlZC0y"]:
        session.web_authn.add_credential(
            authenticator, create_credential(credential_id=credential_id)
        )

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 3

    response = remove_all_credentials(session, authenticator)
    assert_success(response)

    credentials = session.web_authn.get_credentials(authenticator)
    assert len(credentials) == 0
