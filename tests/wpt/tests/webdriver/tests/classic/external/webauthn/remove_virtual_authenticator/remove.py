from tests.support.classic.asserts import assert_success

from . import remove_virtual_authenticator


def test_remove_virtual_authenticator(session):
    authenticator_id = session.web_authn.add_virtual_authenticator(
        {"protocol": "ctap2", "transport": "internal"}
    )

    response = remove_virtual_authenticator(session, authenticator_id)
    assert_success(response)
