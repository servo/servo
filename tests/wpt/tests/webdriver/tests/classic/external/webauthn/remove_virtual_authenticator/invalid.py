import pytest

from tests.support.classic.asserts import assert_error

from . import remove_virtual_authenticator


def test_authenticator_id_invalid_value(session):
    response = remove_virtual_authenticator(session, authenticator_id="invalid")
    assert_error(response, "invalid argument")
