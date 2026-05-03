import pytest
from tests.support.classic.asserts import assert_error

from . import add_virtual_authenticator


def test_empty_parameters(session):
    response = add_virtual_authenticator(session, {})
    assert_error(response, "invalid argument")


def test_protocol_missing(session):
    response = add_virtual_authenticator(session, config={"transport": "internal"})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("protocol", [None, 123, True, [], {}])
def test_protocol_invalid_type(session, protocol):
    config = {
        "protocol": protocol,
        "transport": "internal",
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


def test_protocol_invalid_value(session):
    config = {
        "protocol": "invalid",
        "transport": "internal",
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


def test_transport_missing(session):
    response = add_virtual_authenticator(session, config={"protocol": "ctap2"})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("transport", [None, 123, True, [], {}])
def test_transport_invalid_type(session, transport):
    config = {
        "protocol": "ctap2",
        "transport": transport,
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


def test_transport_invalid_value(session):
    config = {
        "protocol": "ctap2",
        "transport": "invalid",
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("has_resident_key", ["invalid", 123, [], {}])
def test_has_resident_key_invalid_type(session, has_resident_key):
    config = {
        "protocol": "ctap2",
        "transport": "internal",
        "hasResidentKey": has_resident_key,
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("has_user_verification", ["invalid", 123, [], {}])
def test_has_user_verification_invalid_type(session, has_user_verification):
    config = {
        "protocol": "ctap2",
        "transport": "internal",
        "hasUserVerification": has_user_verification,
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("is_user_consenting", ["invalid", 123, [], {}])
def test_is_user_consenting_invalid_type(session, is_user_consenting):
    config = {
        "protocol": "ctap2",
        "transport": "internal",
        "isUserConsenting": is_user_consenting,
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("is_user_verified", ["invalid", 123, [], {}])
def test_is_user_verified_invalid_type(session, is_user_verified):
    config = {
        "protocol": "ctap2",
        "transport": "internal",
        "isUserVerified": is_user_verified,
    }

    response = add_virtual_authenticator(session, config)
    assert_error(response, "invalid argument")
