import pytest


@pytest.fixture
def authenticator(session):
    authenticator_id = session.web_authn.add_virtual_authenticator(
        {"protocol": "ctap2", "transport": "internal", "hasResidentKey": True}
    )

    yield authenticator_id

    session.web_authn.remove_all_credentials(authenticator_id)
    session.web_authn.remove_virtual_authenticator(authenticator_id)
