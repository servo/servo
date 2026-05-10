import pytest

from tests.support.classic.asserts import assert_success
from . import add_virtual_authenticator


@pytest.mark.parametrize(
    "protocol", ["ctap1/u2f", "ctap2", "ctap2_1"]
)
def test_add_virtual_authenticator_protocols(session, protocol):
    config = {
        "protocol": protocol,
        "transport": "internal",
    }

    response = add_virtual_authenticator(session, config)
    authenticator_id = assert_success(response)

    assert isinstance(authenticator_id, str)

    session.web_authn.remove_virtual_authenticator(authenticator_id)


@pytest.mark.parametrize(
    "transport", ["ble", "hybrid", "internal", "nfc", "smart-card", "usb"]
)
def test_add_virtual_authenticator_transports(session, transport):
    config = {
        "protocol": "ctap2",
        "transport": transport,
    }

    response = add_virtual_authenticator(session, config)
    authenticator_id = assert_success(response)

    assert isinstance(authenticator_id, str)

    session.web_authn.remove_virtual_authenticator(authenticator_id)
