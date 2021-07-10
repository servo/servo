# META: timeout=long

import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import document_hidden


def minimize_window(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/minimize".format(**vars(session)))


@pytest.mark.parametrize("i", range(5))
def test_stress(session, i):
    assert not document_hidden(session)
    response = minimize_window(session)
    assert_success(response)
    assert document_hidden(session)
